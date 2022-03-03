//! Parser for the time portion of a date-time expression

use std::str::from_utf8;

use anyhow::anyhow;
use ariadne::{Config, Label, Report, ReportKind, Source};
use bytes::{BufMut, BytesMut};
use logos::{Lexer, Logos, Span};
use logos_iter::{LogosIter, PeekableLexer};

use crate::generators::datetime_expressions::{Adjustment, ClockHour, error, Operation, TimeBase, TimeOffsetType};

/// Struct storing the result of a parsed time expression
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedTimeExpression {
  /// Base to use to start the evaluation
  pub base: TimeBase,
  /// All the adjustments to make to the base
  pub adjustments: Vec<Adjustment<TimeOffsetType>>
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub enum TimeExpressionToken {
  #[token("now")]
  Now,

  #[token("midnight")]
  Midnight,

  #[token("noon")]
  Noon,

  #[token("o\'clock")]
  OClock,

  #[token("am")]
  Am,

  #[token("pm")]
  Pm,

  #[token("+")]
  Plus,

  #[token("-")]
  Minus,

  #[token("next")]
  Next,

  #[token("last")]
  Last,

  #[regex("[0-9]+", |lex| lex.slice().parse())]
  Digits(u64),

  #[regex("hour(s)?")]
  Hours,

  #[regex("minute(s)?")]
  Minutes,

  #[regex("second(s)?")]
  Seconds,

  #[regex("millisecond(s)?")]
  Milliseconds,

  #[error]
  #[regex(r"[ \t\n\f]+", logos::skip)]
  Error
}

impl TimeExpressionToken {
  fn is_base(&self) -> bool {
    match self {
      TimeExpressionToken::Now => true,
      TimeExpressionToken::Midnight => true,
      TimeExpressionToken::Noon => true,
      TimeExpressionToken::Digits(_) => true,
      _ => false
    }
  }

  fn is_op(&self) -> bool {
    match self {
      TimeExpressionToken::Plus => true,
      TimeExpressionToken::Minus => true,
      _ => false
    }
  }
}

// expression returns [ TimeBase timeBase = TimeBase.Now.INSTANCE, List<Adjustment<TimeOffsetType>> adj = new ArrayList<>() ] : ( base { $timeBase = $base.t; }
//     | op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } ( op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } )*
//     | base { $timeBase = $base.t; } ( op duration { if ($duration.d != null) $adj.add($duration.d.withOperation($op.o)); } )*
//     | 'next' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.PLUS)); }
//     | 'next' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.PLUS)); }  ( op duration {
//         if ($duration.d != null) $adj.add($duration.d.withOperation($op.o));
//     } )*
//     | 'last' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.MINUS)); }
//     | 'last' offset { $adj.add(new Adjustment($offset.type, $offset.val, Operation.MINUS)); } ( op duration {
//         if ($duration.d != null) $adj.add($duration.d.withOperation($op.o));
//     } )*
//     ) EOF
//     ;
pub(crate) fn expression<'a>(lex: &'a mut PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>, exp: &str) -> anyhow::Result<ParsedTimeExpression> {
  let mut time_base = TimeBase::Now;
  let mut adj = vec![];

  let mut lex = lex.clone();
  if let Some(token) = lex.peek() {
    if token.is_base() {
      let token = lex.next().unwrap();
      (time_base, lex) = base(lex, exp, token)?;
      if let Some(token) = lex.peek() {
        if token.is_op() {
          let (a, lex2) = adjustments(lex, exp)?;
          adj.extend_from_slice(&*a);
          lex = lex2;
        } else {
          return Err(error(exp, "+ or -", Some(lex.span())));
        }
      }
    } else if token.is_op() {
      let (a, lex2) = adjustments(lex, exp)?;
      adj.extend_from_slice(&*a);
      lex = lex2;
    } else if *token == TimeExpressionToken::Next {
      let _ = lex.next();
      let (t, mut lex2) = offset(lex.clone(), exp)?;
      adj.push(Adjustment {
        adjustment_type: t,
        value: 1,
        operation: Operation::PLUS
      });
      if let Some(token) = lex2.peek() {
        if token.is_op() {
          let (a, lex2) = adjustments(lex2, exp)?;
          adj.extend_from_slice(&*a);
          lex = lex2;
        } else {
          return Err(error(exp, "+ or -", Some(lex2.span())));
        }
      } else {
        lex = lex2.clone();
      }
    } else if *token == TimeExpressionToken::Last {
      let _ = lex.next();
      let (t, mut lex2) = offset(lex.clone(), exp)?;
      adj.push(Adjustment {
        adjustment_type: t,
        value: 1,
        operation: Operation::MINUS
      });
      if let Some(token) = lex2.next() {
        if token.is_op() {
          let (a, lex2) = adjustments(lex2, exp)?;
          adj.extend_from_slice(&*a);
          lex = lex2;
        } else {
          return Err(error(exp, "+ or -", Some(lex2.span())));
        }
      } else {
        lex = lex2;
      }
    } else {
      return Err(error(exp, "one of now, midnight, noon, 1-12 o'clock, +, -, next or last", Some(lex.span())));
    }

    let span = lex.span();
    let remainder = lex.remainder().trim();
    if !remainder.is_empty() {
      Err(error(exp, "no more tokens", Some(span)))
    } else {
      Ok(ParsedTimeExpression {
        base: time_base,
        adjustments: adj
      })
    }
  } else {
    Err(error(exp, "one of now, midnight, noon, 1-12 o'clock, +, -, next or last", None))
  }
}

// base returns [ TimeBase t ] : 'now' { $t = TimeBase.Now.INSTANCE; }
//     | 'midnight' { $t = TimeBase.Midnight.INSTANCE; }
//     | 'noon' { $t = TimeBase.Noon.INSTANCE; }
//     | INT oclock { $t = TimeBase.of($INT.int, $oclock.h); }
//     ;
fn base<'a>(
  mut lex: PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>,
  exp: &str,
  token: TimeExpressionToken
) -> anyhow::Result<(TimeBase, PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>)> {
  match token {
    TimeExpressionToken::Now => Ok((TimeBase::Now, lex.clone())),
    TimeExpressionToken::Midnight => Ok((TimeBase::Midnight, lex.clone())),
    TimeExpressionToken::Noon => Ok((TimeBase::Noon, lex.clone())),
    TimeExpressionToken::Digits(d) => {
      let span = lex.span().clone();
      let mut lex = lex.clone();
      let (hour, lex) = oclock(lex, exp)?;
      TimeBase::of(d, hour, exp, span).map(|t| (t, lex.clone()))
    }
    _ => Err(error(exp, "one of now, midnight, noon or number", Some(lex.span())))
  }
}

// oclock returns [ ClockHour h ] : 'o\'clock' 'am' { $h = ClockHour.AM; }
//     | 'o\'clock' 'pm' { $h = ClockHour.PM; }
//     | 'o\'clock' { $h = ClockHour.NEXT; }
//     ;
fn oclock<'a>(
  mut lex: PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<(ClockHour, PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>)> {
  if let Some(token) = lex.next() {
    if token == TimeExpressionToken::OClock {
      if let Some(next) = lex.peek() {
        if *next == TimeExpressionToken::Am {
          let _ = lex.next();
          Ok((ClockHour::AM, lex.clone()))
        } else if *next == TimeExpressionToken::Pm {
          let _ = lex.next();
          Ok((ClockHour::PM, lex.clone()))
        } else {
          Ok((ClockHour::NEXT, lex.clone()))
        }
      } else {
        Ok((ClockHour::NEXT, lex.clone()))
      }
    } else {
      Err(error(exp, "o\'clock", Some(lex.span())))
    }
  } else {
    Err(error(exp, "o\'clock", Some(lex.span())))
  }
}

// adjustments: op duration ( op duration )*
fn adjustments<'a>(
  mut lex: PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<(Vec<Adjustment<TimeOffsetType>>, PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>)> {
  let mut results = vec![];

  while let Some(token) = lex.peek() {
    if token.is_op() {
      let token = lex.next().unwrap();
      let (adj, lex2) = adjustment(lex, exp, token)?;
      results.push(adj);
      lex = lex2.clone();
    } else {
      break
    }
  }

  Ok((results, lex))
}

fn adjustment<'a>(
  mut lex: PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>,
  exp: &str,
  token: TimeExpressionToken
) -> anyhow::Result<(Adjustment<TimeOffsetType>, PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>)> {
  let (op, lex) = operation(lex, exp, token)?;
  let (adjustment_type, d, lex) = duration(lex, exp)?;
  Ok((Adjustment {
    adjustment_type,
    value: d,
    operation: op
  }, lex.clone()))
}

// duration : INT durationType ;
fn duration<'a>(
  mut lex: PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<(TimeOffsetType, u64, PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>)> {
  if let Some(TimeExpressionToken::Digits(n)) = lex.next() {
    let (tot, lex) = duration_type(lex, exp)?;
    Ok((tot, n, lex.clone()))
  } else {
    Err(error(exp, "an integer value", Some(lex.span())))
  }
}

// durationType returns [ TimeOffsetType type ] : 'hour' { $type = TimeOffsetType.HOUR; }
//     | HOURS { $type = TimeOffsetType.HOUR; }
//     | 'minute' { $type = TimeOffsetType.MINUTE; }
//     | MINUTES { $type = TimeOffsetType.MINUTE; }
//     | 'second' { $type = TimeOffsetType.SECOND; }
//     | SECONDS { $type = TimeOffsetType.SECOND; }
//     | 'millisecond' { $type = TimeOffsetType.MILLISECOND; }
//     | MILLISECONDS { $type = TimeOffsetType.MILLISECOND; }
//     ;
fn duration_type<'a>(
  mut lex: PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<(TimeOffsetType, PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>)> {
  if let Some(token) = lex.next() {
    match token {
      TimeExpressionToken::Hours => Ok((TimeOffsetType::HOUR, lex.clone())),
      TimeExpressionToken::Minutes => Ok((TimeOffsetType::MINUTE, lex.clone())),
      TimeExpressionToken::Seconds => Ok((TimeOffsetType::SECOND, lex.clone())),
      TimeExpressionToken::Milliseconds => Ok((TimeOffsetType::MILLISECOND, lex.clone())),
      _ => Err(error(exp, "a duration type (hour(s), minute(s), etc.)", Some(lex.span())))
    }
  } else {
    Err(error(exp, "a duration type (hour(s), minute(s), etc.)", Some(lex.span())))
  }
}

// op returns [ Operation o ] : '+' { $o = Operation.PLUS; }
//     | '-' { $o = Operation.MINUS; }
//     ;
fn operation<'a>(
  mut lex: PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>,
  exp: &str,
  token: TimeExpressionToken
) -> anyhow::Result<(Operation, PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>)> {
  match token {
    TimeExpressionToken::Plus => Ok((Operation::PLUS, lex.clone())),
    TimeExpressionToken::Minus => Ok((Operation::MINUS, lex.clone())),
    _ => Err(error(exp, "+ or -", Some(lex.span())))
  }
}

// offset returns [ TimeOffsetType type, int val = 1 ] : 'hour' { $type = TimeOffsetType.HOUR; }
//     | 'minute' { $type = TimeOffsetType.MINUTE; }
//     | 'second' { $type = TimeOffsetType.SECOND; }
//     | 'millisecond' { $type = TimeOffsetType.MILLISECOND; }
//     ;
fn offset<'a>(
  mut lex: PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>,
  exp: &str
) -> anyhow::Result<(TimeOffsetType, PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>)> {
  if let Some(token) = lex.next() {
    match token {
      TimeExpressionToken::Hours => Ok((TimeOffsetType::HOUR, lex)),
      TimeExpressionToken::Minutes => Ok((TimeOffsetType::MINUTE, lex)),
      TimeExpressionToken::Seconds => Ok((TimeOffsetType::SECOND, lex)),
      TimeExpressionToken::Milliseconds => Ok((TimeOffsetType::MILLISECOND, lex)),
      _ => Err(error(exp, "an offset type (hour, minute, second, etc.)", Some(lex.span())))
    }
  } else {
    Err(error(exp, "an offset type (hour, minute, second, etc.)", Some(lex.span())))
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use logos::Logos;
  use logos_iter::{LogosIter, PeekableLexer};
  use trim_margin::MarginTrimmable;

  use crate::generators::datetime_expressions::{Adjustment, DateBase, DateOffsetType, Operation, TimeBase, TimeOffsetType};
  use crate::generators::time_expression_parser::ParsedTimeExpression;

  macro_rules! as_string {
    ($e:expr) => {{ $e.map_err(|err| err.to_string()) }};
  }

  #[test]
  fn invalid_expression() {
    let mut lex = super::TimeExpressionToken::lexer("not valid").peekable_lexer();
    expect!(as_string!(super::expression(&mut lex, "not valid"))).to(
      be_err().value(
        "|Error: Expected one of now, midnight, noon, 1-12 o'clock, +, -, next or last
            |   ╭─[expression:1:1]
            |   │
            | 1 │ not valid
            |   · ┬ \u{0020}
            |   · ╰── Expected one of now, midnight, noon, 1-12 o'clock, +, -, next or last here
            |───╯
            |
            ".trim_margin_with("|").unwrap()
      ));

    let mut lex = super::TimeExpressionToken::lexer("44 o'clock").peekable_lexer();
    expect!(as_string!(super::expression(&mut lex, "44 o'clock"))).to(
      be_err().value(
        "|Error: Expected hour 1 to 12
            |   ╭─[expression:1:1]
            |   │
            | 1 │ 44 o'clock
            |   · ─┬ \u{0020}
            |   ·  ╰── Expected hour 1 to 12 here
            |───╯
            |
            ".trim_margin_with("|").unwrap()
      ));

    let mut lex = super::TimeExpressionToken::lexer("now today not valid").peekable_lexer();
    expect!(as_string!(super::expression(&mut lex, "now today not valid"))).to(
      be_err().value(
        "|Error: Expected + or -
            |   ╭─[expression:1:5]
            |   │
            | 1 │ now today not valid
            |   ·     ┬ \u{0020}
            |   ·     ╰── Expected + or - here
            |───╯
            |
            ".trim_margin_with("|").unwrap()
      ));
  }

  #[test]
  fn base_only() {
    let mut lex = super::TimeExpressionToken::lexer("now").peekable_lexer();
    expect!(super::expression(&mut lex, "now")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Now,
      adjustments: vec![]
    }));

    let mut lex = super::TimeExpressionToken::lexer("  midnight   ").peekable_lexer();
    expect!(super::expression(&mut lex, "  midnight   ")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Midnight,
      adjustments: vec![]
    }));

    let mut lex = super::TimeExpressionToken::lexer("noon").peekable_lexer();
    expect!(super::expression(&mut lex, "noon")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Noon,
      adjustments: vec![]
    }));

    let mut lex = super::TimeExpressionToken::lexer("1 o'clock").peekable_lexer();
    expect!(super::expression(&mut lex, "1 o'clock")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Next { hour: 1 },
      adjustments: vec![]
    }));
  }

  #[test]
  fn op_and_duration() {
    let mut lex = super::TimeExpressionToken::lexer("+1 hour").peekable_lexer();
    expect!(super::expression(&mut lex, "+1 hour")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Now,
      adjustments: vec![
        Adjustment {
          adjustment_type: TimeOffsetType::HOUR,
          value: 1,
          operation: Operation::PLUS
        }
      ]
    }));

    let mut lex = super::TimeExpressionToken::lexer("+ 2 hours - 1 second").peekable_lexer();
    expect!(super::expression(&mut lex, "+ 2 hours - 1 second")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Now,
      adjustments: vec![
        Adjustment {
          adjustment_type: TimeOffsetType::HOUR,
          value: 2,
          operation: Operation::PLUS
        },
        Adjustment {
          adjustment_type: TimeOffsetType::SECOND,
          value: 1,
          operation: Operation::MINUS
        }
      ]
    }));
  }

  #[test]
  fn base_and_op_and_duration() {
    let mut lex = super::TimeExpressionToken::lexer("midnight + 2 hours").peekable_lexer();
    expect!(super::expression(&mut lex, "midnight + 2 hours")).to(be_ok().value(ParsedTimeExpression {
      base: TimeBase::Midnight,
      adjustments: vec![
        Adjustment {
          adjustment_type: TimeOffsetType::HOUR,
          value: 2,
          operation: Operation::PLUS
        }
      ]
    }));

    let mut lex = super::TimeExpressionToken::lexer("midnight 2 week").peekable_lexer();
    expect!(as_string!(super::expression(&mut lex, "midnight 2 week"))).to(
      be_err().value(
        "|Error: Expected + or -
            |   ╭─[expression:1:10]
            |   │
            | 1 │ midnight 2 week
            |   ·          ┬ \u{0020}
            |   ·          ╰── Expected + or - here
            |───╯
            |
            ".trim_margin_with("|").unwrap()
      ));
  }
}
