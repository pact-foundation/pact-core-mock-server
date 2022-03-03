//! Parser for the time portion of a date-time expression

use std::str::from_utf8;

use anyhow::anyhow;
use ariadne::{Config, Label, Report, ReportKind, Source};
use bytes::{BufMut, BytesMut};
use logos::{Lexer, Logos, Span};
use logos_iter::{LogosIter, PeekableLexer};

use crate::generators::datetime_expressions::{Adjustment, ClockHour, error, TimeBase, TimeOffsetType};

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

  #[regex("[0-9]+", |lex| lex.slice().parse())]
  Digits(u64),

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

  if let Some(token) = lex.next() {
    if token.is_base() {
      time_base = base(lex, exp, &token)?;
      // if let Some(token) = lex.next() {
      //   if token.is_op() {
      //     adj.extend_from_slice(&*adjustments(lex, exp, &token)?);
      //   } else {
      //     return Err(error(exp, "+ or -", Some(lex.span())));
      //   }
      // }
    // } else if token.is_op() {
    //   adj.extend_from_slice(&*adjustments(lex, exp, &token)?);
    // } else if token == TimeExpressionToken::Next {
    //   let (t, v) = offset(lex, exp)?;
    //   adj.push(Adjustment {
    //     adjustment_type: t,
    //     value: v,
    //     operation: Operation::PLUS
    //   });
    //   if let Some(token) = lex.next() {
    //     if token.is_op() {
    //       adj.extend_from_slice(&*adjustments(lex, exp, &token)?);
    //     } else {
    //       return Err(error(exp, "+ or -", Some(lex.span())));
    //     }
    //   }
    // } else if token == TimeExpressionToken::Last {
    //   let (t, v) = offset(lex, exp)?;
    //   adj.push(Adjustment {
    //     adjustment_type: t,
    //     value: v,
    //     operation: Operation::MINUS
    //   });
    //   if let Some(token) = lex.next() {
    //     if token.is_op() {
    //       adj.extend_from_slice(&*adjustments(lex, exp, &token)?);
    //     } else {
    //       return Err(error(exp, "+ or -", Some(lex.span())));
    //     }
    //   }
    } else {
      return Err(error(exp, "one of now, midnight, noon, 1-12 o'clock, +, -, next or last", Some(lex.span())));
    }

    // let span = lex.clone().span();
    // let remainder = lex.peek_remainder().trim();
    // if !remainder.is_empty() {
    //   Err(error(exp, "no more tokens", Some(span)))
    // } else {
      Ok(ParsedTimeExpression {
        base: time_base,
        adjustments: adj
      })
    // }
  } else {
    Err(error(exp, "one of now, midnight, noon, 1-12 o'clock, +, -, next or last", None))
  }
}

// base returns [ TimeBase t ] : 'now' { $t = TimeBase.Now.INSTANCE; }
//     | 'midnight' { $t = TimeBase.Midnight.INSTANCE; }
//     | 'noon' { $t = TimeBase.Noon.INSTANCE; }
//     | INT oclock { $t = TimeBase.of($INT.int, $oclock.h); }
//     ;
fn base<'a>(lex: &'a mut PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>, exp: &str, token: &TimeExpressionToken) -> anyhow::Result<TimeBase> {
  match token {
    TimeExpressionToken::Now => Ok(TimeBase::Now),
    TimeExpressionToken::Midnight => Ok(TimeBase::Midnight),
    TimeExpressionToken::Noon => Ok(TimeBase::Noon),
    TimeExpressionToken::Digits(d) => {
      let span = lex.span();
      let hour = oclock(lex, exp)?;
      TimeBase::of(*d, hour, exp, span)
    }
    _ => Err(error(exp, "one of now, midnight, noon or number", Some(lex.span())))
  }
}

// oclock returns [ ClockHour h ] : 'o\'clock' 'am' { $h = ClockHour.AM; }
//     | 'o\'clock' 'pm' { $h = ClockHour.PM; }
//     | 'o\'clock' { $h = ClockHour.NEXT; }
//     ;
fn oclock<'a>(lex: &'a mut PeekableLexer<'a, Lexer<'a, TimeExpressionToken>, TimeExpressionToken>, exp: &str) -> anyhow::Result<ClockHour> {
  if let Some(token) = lex.next() {
    if token == TimeExpressionToken::OClock {
      if let Some(next) = lex.peek() {
        if *next == TimeExpressionToken::Am {
          let _ = lex.next();
          Ok(ClockHour::AM)
        } else if *next == TimeExpressionToken::Pm {
          let _ = lex.next();
          Ok(ClockHour::PM)
        } else {
          Ok(ClockHour::NEXT)
        }
      } else {
        Ok(ClockHour::NEXT)
      }
    } else {
      Err(error(exp, "o\'clock", Some(lex.span())))
    }
  } else {
    Err(error(exp, "o\'clock", Some(lex.span())))
  }
}

// duration returns [ Adjustment<TimeOffsetType> d ] : INT durationType { $d = new Adjustment<TimeOffsetType>($durationType.type, $INT.int); } ;
//
// durationType returns [ TimeOffsetType type ] : 'hour' { $type = TimeOffsetType.HOUR; }
//     | HOURS { $type = TimeOffsetType.HOUR; }
//     | 'minute' { $type = TimeOffsetType.MINUTE; }
//     | MINUTES { $type = TimeOffsetType.MINUTE; }
//     | 'second' { $type = TimeOffsetType.SECOND; }
//     | SECONDS { $type = TimeOffsetType.SECOND; }
//     | 'millisecond' { $type = TimeOffsetType.MILLISECOND; }
//     | MILLISECONDS { $type = TimeOffsetType.MILLISECOND; }
//     ;
//
// op returns [ Operation o ] : '+' { $o = Operation.PLUS; }
//     | '-' { $o = Operation.MINUS; }
//     ;
//
// offset returns [ TimeOffsetType type, int val = 1 ] : 'hour' { $type = TimeOffsetType.HOUR; }
//     | 'minute' { $type = TimeOffsetType.MINUTE; }
//     | 'second' { $type = TimeOffsetType.SECOND; }
//     | 'millisecond' { $type = TimeOffsetType.MILLISECOND; }
//     ;
//
// HOURS : 'hour' 's'? ;
// SECONDS : 'second' 's'? ;
// MINUTES : 'minute' 's'? ;
// MILLISECONDS : 'millisecond' 's'? ;

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use logos::Logos;
  use logos_iter::{LogosIter, PeekableLexer};
  use trim_margin::MarginTrimmable;

  use crate::generators::datetime_expressions::{Adjustment, DateBase, DateOffsetType, Operation, TimeBase};
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
            |   ·     ──┬── \u{0020}
            |   ·       ╰──── Expected + or - here
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

  // #[test]
  // fn op_and_duration() {
  //   let mut lex = super::TimeExpressionToken::lexer("+1 day").peekable_lexer();
  //   expect!(super::expression(&mut lex, "+1 day")).to(be_ok().value(ParsedTimeExpression {
  //     base: DateBase::NOW,
  //     adjustments: vec![
  //       Adjustment {
  //         adjustment_type: DateOffsetType::DAY,
  //         value: 1,
  //         operation: Operation::PLUS
  //       }
  //     ]
  //   }));
  //
  //   let mut lex = super::TimeExpressionToken::lexer("+ 2 weeks - 1 day").peekable_lexer();
  //   expect!(super::expression(&mut lex, "+ 2 weeks - 1 day")).to(be_ok().value(ParsedTimeExpression {
  //     base: DateBase::NOW,
  //     adjustments: vec![
  //       Adjustment {
  //         adjustment_type: DateOffsetType::WEEK,
  //         value: 2,
  //         operation: Operation::PLUS
  //       },
  //       Adjustment {
  //         adjustment_type: DateOffsetType::DAY,
  //         value: 1,
  //         operation: Operation::MINUS
  //       }
  //     ]
  //   }));
  // }
  //
  // #[test]
  // fn base_and_op_and_duration() {
  //   let mut lex = super::TimeExpressionToken::lexer("today + 2 week").peekable_lexer();
  //   expect!(super::expression(&mut lex, "today + 2 week")).to(be_ok().value(ParsedTimeExpression {
  //     base: DateBase::TODAY,
  //     adjustments: vec![
  //       Adjustment {
  //         adjustment_type: DateOffsetType::WEEK,
  //         value: 2,
  //         operation: Operation::PLUS
  //       }
  //     ]
  //   }));
  //
  //   let mut lex = super::TimeExpressionToken::lexer("today 2 week").peekable_lexer();
  //   expect!(as_string!(super::expression(&mut lex, "today 2 week"))).to(
  //     be_err().value(
  //       "|Error: Expected + or -
  //           |   ╭─[expression:1:7]
  //           |   │
  //           | 1 │ today 2 week
  //           |   ·       ┬ \u{0020}
  //           |   ·       ╰── Expected + or - here
  //           |───╯
  //           |
  //           ".trim_margin_with("|").unwrap()
  //     ));
  // }
}
