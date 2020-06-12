//! Utilities for dealing with time and date values. These are based on the Java DateTimeFormatter
//!
//!   Symbol  Meaning                     Presentation      Examples
//!   ------  -------                     ------------      -------
//!    G       era                         text              AD; Anno Domini; A
//!    u       year                        year              2004; 04
//!    y       year-of-era                 year              2004; 04
//!    D       day-of-year                 number            189
//!    M/L     month-of-year               number/text       7; 07; Jul; July; J
//!    d       day-of-month                number            10
//!    Q/q     quarter-of-year             number/text       3; 03; Q3; 3rd quarter
//!    Y       week-based-year             year              1996; 96
//!    w       week-of-week-based-year     number            27
//!    W       week-of-month               number            4
//!    E       day-of-week                 text              Tue; Tuesday; T
//!    e/c     localized day-of-week       number/text       2; 02; Tue; Tuesday; T
//!    F       week-of-month               number            3
//!    a       am-pm-of-day                text              PM
//!    h       clock-hour-of-am-pm (1-12)  number            12
//!    K       hour-of-am-pm (0-11)        number            0
//!    k       clock-hour-of-am-pm (1-24)  number            0
//!    H       hour-of-day (0-23)          number            0
//!    m       minute-of-hour              number            30
//!    s       second-of-minute            number            55
//!    S       fraction-of-second          fraction          978
//!    A       milli-of-day                number            1234
//!    n       nano-of-second              number            987654321
//!    N       nano-of-day                 number            1234000000

//!    V       time-zone ID                zone-id           America/Los_Angeles; Z; -08:30
//!    z       time-zone name              zone-name         Pacific Standard Time; PST
//!    O       localized zone-offset       offset-O          GMT+8; GMT+08:00; UTC-08:00;
//!    X       zone-offset 'Z' for zero    offset-X          Z; -08; -0830; -08:30; -083015; -08:30:15;
//!    x       zone-offset                 offset-x          +0000; -08; -0830; -08:30; -083015; -08:30:15;
//!    Z       zone-offset                 offset-Z          +0000; -0800; -08:00;
//!    '       escape for text             delimiter
//!    ''      single quote                literal           '

use std::fmt::{Display, Formatter};

use chrono::Local;
use itertools::Itertools;
use log::*;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, tag_no_case, take_while_m_n};
use nom::character::complete::{char, digit1, alphanumeric1};
use nom::combinator::{value, opt};
use nom::Err::{Error, Failure};
use nom::error::{ErrorKind, ParseError};
use nom::IResult;
use nom::multi::many1;
use nom::sequence::{delimited, preceded, terminated, tuple, separated_pair};
use crate::timezone_db::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
/// Tokens for DateTime patterns
pub enum DateTimePatternToken {
  Era(usize),
  Year(usize),
  Month(usize),
  MonthNum(usize),
  Text(String),
  WeekInYear,
  WeekInMonth(bool),
  DayInYear,
  DayInMonth,
  DayName(usize),
  DayOfWeek(usize),
  AmPm,
  Hour24,
  Hour24ZeroBased,
  Hour12,
  Hour12ZeroBased,
  Minute,
  Second,
  Millisecond(usize),
  Nanosecond(usize),
  TimezoneOffset(usize),
  TimezoneOffsetX(usize),
  TimezoneOffsetXZZero(usize),
  TimezoneOffsetGmt(usize),
  TimezoneName(usize),
  TimezoneId(usize),
  QuarterOfYear(usize),
  QuarterOfYearNum(usize),
  MillisecondOfDay,
  NanosecondOfDay
}

#[derive(Debug, PartialEq, Clone)]
/// Errors when parsing a date time pattern
pub enum DateTimePatternError<I> {
  /// Too many pattern letters were encountered
  TooManyPatternLetters(String, usize),
  /// Unparsed characters remaining
  RemainingCharacters(String),
  /// Nom error occurred
  Nom(I, ErrorKind),
}

impl<I> ParseError<I> for DateTimePatternError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    DateTimePatternError::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

impl <I> Display for DateTimePatternError<I> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DateTimePatternError::TooManyPatternLetters(s, _count)  => f.write_str(s),
      DateTimePatternError::RemainingCharacters(s) => f.write_fmt(format_args!("Remaining unmatched characters at '{}'", s)),
      DateTimePatternError::Nom(_s, err) => f.write_str(err.description())
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
#[allow(missing_docs)]
/// Errors when parsing date time values
pub enum DateTimeError<I> {
  InvalidDayInYear(String),
  InvalidDayInMonth(String),
  InvalidMonth(String),
  InvalidQuarter(String),
  InvalidWeekInYear(String),
  InvalidDayOfWeek(String),
  InvalidHour(String),
  InvalidMinute(String),
  InvalidMillisecond(String),
  FullTimezonesNotSupported(String),
  InvalidTimezone(String),
  /// Nom error occurred
  Nom(I, ErrorKind),
}

impl<I> ParseError<I> for DateTimeError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    DateTimeError::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

fn is_digit(ch: char) -> bool {
  ch.is_ascii_digit()
}

fn is_uppercase(ch: char) -> bool {
  ch.is_ascii_uppercase()
}

fn validate_number(m: &str, num_type: String, lower: usize, upper: usize) -> Result<&str, String> {
  match m.parse::<usize>() {
    Ok(v) => if v >= lower && v <= upper {
      Ok(m)
    } else {
      Err(format!("Invalid {} {}", num_type, v))
    },
    Err(err) => Err(format!("{}", err))
  }
}

fn era_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("G")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 5 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Era ('G'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::Era(result.len())))
      }
    })
}

fn ampm_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("a")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 1 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for AM/PM ('a'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::AmPm))
      }
    })
}

fn week_in_year_month_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("w"), is_a("W"), is_a("F")))(s)
    .and_then(|(remaining, result)| {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Week in Month ('W' or 'F'): {}", result.len()), result.len())))
      } else if result.starts_with("w") {
        Ok((remaining, DateTimePatternToken::WeekInYear))
      } else {
        Ok((remaining, DateTimePatternToken::WeekInMonth(result.starts_with('W'))))
      }
    })
}

fn day_in_year_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("D")(s).map(|(remaining, _)| {
    (remaining, DateTimePatternToken::DayInYear)
  })
}

fn day_in_month_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("d")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Day in Month ('d'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::DayInMonth))
      }
    })
}

fn day_name_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("E")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 5 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Day of Week ('E'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::DayName(result.len())))
      }
    })
}

fn day_of_week_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("e"), is_a("c")))(s)
    .and_then(|(remaining, result)| {
      if result.len() > 5 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Day of Week ('e'): {}", result.len()), result.len())))
      } else if result.starts_with('c') && result.len() > 1 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Day of Week ('c'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::DayOfWeek(result.len())))
      }
    })
}

fn year_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("y"), is_a("Y"), is_a("u")))(s).map(|(remaining, result)| {
    (remaining, DateTimePatternToken::Year(result.len()))
  })
}

fn month_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("M"), is_a("L")))(s).and_then(|(remaining, result)| {
    if result.len() > 5 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Month ('M' or 'L'): {}", result.len()), result.len())))
    } else if result.starts_with('M') {
      Ok((remaining, DateTimePatternToken::Month(result.len())))
    } else {
      Ok((remaining, DateTimePatternToken::MonthNum(result.len())))
    }
  })
}

fn quarter_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("q"), is_a("Q")))(s).and_then(|(remaining, result)| {
    if result.len() > 5 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Quarter ('q' or 'Q'): {}", result.len()), result.len())))
    } else if result.starts_with('Q') {
      Ok((remaining, DateTimePatternToken::QuarterOfYear(result.len())))
    } else if result.starts_with('q') && result.len() > 2 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Quarter ('q'): {}", result.len()), result.len())))
    } else {
      Ok((remaining, DateTimePatternToken::QuarterOfYearNum(result.len())))
    }
  })
}

fn quoted_text_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  delimited(char('\''), many1(alt((tag("''"), is_not("'")))), char('\''))(s)
    .map(|(remaining, result)| {
      (remaining, DateTimePatternToken::Text(result.join("").chars().coalesce(|x, y| {
        if x == '\'' && y == '\'' { Ok('\'') } else { Err((x, y)) }
      }).collect()))
    })
}

fn quote_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  value(DateTimePatternToken::Text("'".into()), tag("''"))(s)
}

fn hour_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("h"), is_a("H"), is_a("k"), is_a("K")))(s).and_then(|(remaining, result)| {
    if result.len() > 2 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Hour ('h', 'H', 'k' or 'K'): {}", result.len()), result.len())))
    } else if result.starts_with('h') {
      Ok((remaining, DateTimePatternToken::Hour12))
    } else if result.starts_with('H') {
      Ok((remaining, DateTimePatternToken::Hour24ZeroBased))
    } else if result.starts_with('k') {
      Ok((remaining, DateTimePatternToken::Hour24))
    } else {
      Ok((remaining, DateTimePatternToken::Hour12ZeroBased))
    }
  })
}

fn minute_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("m")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Minute ('m'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::Minute))
      }
    })
}

fn second_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("s")(s)
    .and_then(|(remaining, result)| {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Minute ('m'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::Second))
      }
    })
}

fn millisecond_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("S")(s)
    .and_then(|(remaining, result)| {
      Ok((remaining, DateTimePatternToken::Millisecond(result.len())))
    })
}

fn nanosecond_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_a("n")(s)
      .and_then(|(remaining, result)| {
        Ok((remaining, DateTimePatternToken::Nanosecond(result.len())))
      })
}

fn millisecond_of_day_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  value(DateTimePatternToken::MillisecondOfDay, is_a("A"))(s)
}

fn nanosecond_of_day_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  value(DateTimePatternToken::NanosecondOfDay, is_a("N"))(s)
}

fn timezone_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  alt((is_a("x"), is_a("X"), is_a("Z"), is_a("O"), is_a("z"), is_a("V")))(s).and_then(|(remaining, result)| {
    if result.len() > 5 {
      Err(Failure(DateTimePatternError::TooManyPatternLetters(
        format!("Too many pattern letters for Timezone Offset ('x', 'X', 'O', 'z', or 'Z'): {}", result.len()), result.len())))
    } else if result.starts_with('Z') {
      Ok((remaining, DateTimePatternToken::TimezoneOffset(result.len())))
    } else if result.starts_with('x') {
      Ok((remaining, DateTimePatternToken::TimezoneOffsetX(result.len())))
    } else if result.starts_with('O') {
      if result.len() > 4 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Timezone Offset ('O'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::TimezoneOffsetGmt(result.len())))
      }
    } else if result.starts_with('V') {
      if result.len() > 2 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Timezone ID ('V'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::TimezoneId(result.len())))
      }
    } else if result.starts_with('z') {
      if result.len() > 4 {
        Err(Failure(DateTimePatternError::TooManyPatternLetters(
          format!("Too many pattern letters for Timezone Offset ('z'): {}", result.len()), result.len())))
      } else {
        Ok((remaining, DateTimePatternToken::TimezoneName(result.len())))
      }
    } else {
      Ok((remaining, DateTimePatternToken::TimezoneOffsetXZZero(result.len())))
    }
  })
}

fn text_pattern(s: &str) -> IResult<&str, DateTimePatternToken, DateTimePatternError<&str>> {
  is_not("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'{}[]")(s).map(|(remaining, result)| {
    (remaining, DateTimePatternToken::Text(result.to_string()))
  })
}

fn era(s: &str, _count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  alt((tag_no_case("ad"), tag_no_case("bc")))(s)
    .map(|(remaining, result)| (remaining, result.into()))
}

fn ampm(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  alt((tag_no_case("am"), tag_no_case("pm")))(s)
    .map(|(remaining, result)| (remaining, result.into()))
}

fn year(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, count, is_digit)(s)
    .map(|(remaining, result)| (remaining, result.into()))
}

fn month_num(s: &str, _count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "month".into(), 1, 12) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidMonth(result.to_string())))
    }
  })
}

fn month(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if count <= 2 {
    month_num(s, count)
  } else if count == 3 {
    alt((
      tag_no_case("jan"),
      tag_no_case("feb"),
      tag_no_case("mar"),
      tag_no_case("apr"),
      tag_no_case("may"),
      tag_no_case("jun"),
      tag_no_case("jul"),
      tag_no_case("aug"),
      tag_no_case("sep"),
      tag_no_case("oct"),
      tag_no_case("nov"),
      tag_no_case("dec"),
    ))(s).map(|(remaining, result)| (remaining, result.into()))
  } else {
    alt((
      tag_no_case("january"),
      tag_no_case("february"),
      tag_no_case("march"),
      tag_no_case("april"),
      tag_no_case("may"),
      tag_no_case("june"),
      tag_no_case("july"),
      tag_no_case("august"),
      tag_no_case("september"),
      tag_no_case("october"),
      tag_no_case("november"),
      tag_no_case("december"),
    ))(s).map(|(remaining, result)| (remaining, result.into()))
  }
}

fn week_in_year(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "week in year".into(), 1, 56) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidWeekInYear(result.to_string())))
    }
  })
}

fn week_in_month(s: &str, from_one: bool) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    let lower = if from_one { 1 } else { 0 };
    let upper = if from_one { 5 } else { 4 };
    match validate_number(result, "week in month".into(), lower, upper) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidWeekInYear(result.to_string())))
    }
  })
}

fn day_in_year(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 3, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "day in year".into(), 1, 356) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidDayInYear(result.to_string())))
    }
  })
}

fn day_in_month(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "day in month".into(), 1, 31) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidDayInMonth(result.to_string())))
    }
  })
}

fn day_of_week(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if count > 2 {
    day_of_week_name(s, count)
  } else {
    take_while_m_n(1, 1, is_digit)(s).and_then(|(remaining, result)|{
      match validate_number(result, "day of week".into(), 1, 7) {
        Ok(_) => Ok((remaining, result.into())),
        Err(_err) => Err(Error(DateTimeError::InvalidDayOfWeek(result.to_string())))
      }
    })
  }
}

fn hour_24(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "hour".into(), 1, 24) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidHour(result.to_string())))
    }
  })
}

fn hour_24_0(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "hour (zero-based)".into(), 0, 23) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidHour(result.to_string())))
    }
  })
}
fn hour_12(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "hour".into(), 1, 12) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidHour(result.to_string())))
    }
  })
}

fn hour_12_0(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "hour (zero-based)".into(), 0, 11) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidHour(result.to_string())))
    }
  })
}

fn minute(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "minute".into(), 0, 59) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidMinute(result.to_string())))
    }
  })
}

fn second(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "second".into(), 0, 60) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidMinute(result.to_string())))
    }
  })
}

fn millisecond(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, count, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "millisecond".into(), 0, 999) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidMillisecond(result.to_string())))
    }
  })
}

fn timezone_hour_min(s: &str) -> IResult<&str, &str, DateTimeError<&str>> {
  tuple((is_a("+-"), hour_12_0, tag(":"), minute))(s)
    .map(|(remaining, _result)| {
      (remaining, "")
    })
}

fn timezone_long_offset(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  match d {
    1 => preceded(is_a("+-"), tuple((hour_12_0, opt(minute))))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1.unwrap_or("".to_string()))
        }),
    2 => preceded(is_a("+-"), tuple((hour_12_0, minute)))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1)
        }),
    3 => preceded(is_a("+-"), tuple((hour_12_0, tag(":"), minute)))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1 + &result.2)
        }),
    4 => preceded(is_a("+-"), tuple((hour_12_0, minute, opt(second))))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1 + &result.2.unwrap_or("".to_string()))
        }),
    _ => preceded(is_a("+-"), tuple((hour_12_0, tag(":"), minute, opt(tuple((tag(":"), second))))))(s)
        .map(|(remaining, result)| {
          let seconds = match &result.3 {
            Some((c, s)) => c.to_string() + s,
            None => "".to_string()
          };
          (remaining, result.0 + &result.1 + &result.2 + &seconds)
        })
  }
}

fn timezone_long_offset_with_z(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  match tag::<&str, &str, DateTimeError<&str>>("Z")(s) {
    Ok((remaining, result)) => Ok((remaining, result.into())),
    Err(_) => timezone_long_offset(s, d)
  }
}

fn timezone(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if d < 4 {
    take_while_m_n(3, 4, is_uppercase)(s).and_then(|(remaining, result)| {
      if validate_tz_abbreviation(result) {
        Ok((remaining, result.into()))
      } else {
        Err(Error(DateTimeError::InvalidTimezone(result.to_string())))
      }
    })
  } else {
    Err(Error(DateTimeError::FullTimezonesNotSupported(s.to_string())))
  }
}

fn timezone_id(s: &str) -> IResult<&str, String, DateTimeError<&str>> {
  separated_pair(alphanumeric1, char('/'), alphanumeric1)(s).and_then(|(remaining, result)| {
    let tz = format!("{}/{}", result.0, result.1);
    if ZONES.contains(tz.as_str()) {
      Ok((remaining, tz.clone()))
    } else {
      Err(Error(DateTimeError::InvalidTimezone(tz.clone())))
    }
  })
}

fn timezone_offset(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  match d {
    1..=3 => preceded(is_a("+-"), tuple((hour_12_0, minute)))(s)
        .map(|(remaining, result)| {
          (remaining, result.0 + &result.1)
        }),
    4 => alt((
          preceded(alt((tag("GMT"), tag("UTC"))), timezone_hour_min),
          tag("GMT"),
          tag("UTC")
        ))(s)
        .map(|(remaining, result)| {
          (remaining, result.into())
        }),
    _ => timezone_hour_min(s)
        .map(|(remaining, result)| {
          (remaining, result.into())
        })
  }
}

fn timezone_offset_gmt(s: &str, d: usize) -> IResult<&str, String, DateTimeError<&str>> {
  match d {
    1..=3 => preceded(alt((tag("GMT"), tag("UTC"))), tuple((is_a("+-"), hour_12_0, opt(preceded(tag(":"), minute)))))(s)
        .map(|(remaining, result)| {
          let minute = match result.2 {
            Some(result) => result,
            None => "".to_string()
          };
          (remaining, result.1 + &minute)
        }),
    _ => preceded(alt((tag("GMT"), tag("UTC"))), tuple((is_a("+-"), hour_12_0, tag(":"), minute)))(s)
        .map(|(remaining, result)| {
          (remaining, result.1 + &result.2 + &result.3)
        })
  }
}

fn day_of_week_name(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if count <= 3 {
    alt((
      tag_no_case("mon"),
      tag_no_case("tue"),
      tag_no_case("wed"),
      tag_no_case("thu"),
      tag_no_case("fri"),
      tag_no_case("sat"),
      tag_no_case("sun")
    ))(s)
  } else {
    alt((
      tag_no_case("monday"),
      tag_no_case("tuesday"),
      tag_no_case("wednesday"),
      tag_no_case("thursday"),
      tag_no_case("friday"),
      tag_no_case("saturday"),
      tag_no_case("sunday")
    ))(s)
  }.map(|(remaining, result)| (remaining, result.into()))
}

fn quarter_num(s: &str, _count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  take_while_m_n(1, 2, is_digit)(s).and_then(|(remaining, result)|{
    match validate_number(result, "quarter".into(), 1, 4) {
      Ok(_) => Ok((remaining, result.into())),
      Err(_err) => Err(Error(DateTimeError::InvalidQuarter(result.to_string())))
    }
  })
}

fn quarter(s: &str, count: usize) -> IResult<&str, String, DateTimeError<&str>> {
  if count < 3 {
    quarter_num(s, count)
  } else if count == 3 {
    alt((tag_no_case("Q1"), tag_no_case("Q2"), tag_no_case("Q3"), tag_no_case("Q4")))(s)
      .map(|(remaining, result)| (remaining, result.into()))
  } else {
    terminated(alt((
      tag_no_case("1st"), tag_no_case("2nd"), tag_no_case("3rd"), tag_no_case("4th")
    )), tag_no_case(" quarter"))(s)
      .map(|(remaining, result)| (remaining, result.into()))
  }
}

/// Parses a string into a vector of date/time tokens
pub fn parse_pattern(s: &str) -> Result<Vec<DateTimePatternToken>, String> {
  match many1(alt((
    era_pattern,
    year_pattern,
    month_pattern,
    day_in_year_pattern,
    day_in_month_pattern,
    quarter_pattern,
    week_in_year_month_pattern,
    day_name_pattern,
    day_of_week_pattern,
    ampm_pattern,
    hour_pattern,
    minute_pattern,
    second_pattern,
    millisecond_pattern,
    nanosecond_pattern,
    millisecond_of_day_pattern,
    nanosecond_of_day_pattern,
    quoted_text_pattern,
    quote_pattern,
    timezone_pattern,
    text_pattern
  )))(s) {
    Ok((remaining, result)) => if !remaining.is_empty() {
      let error = format!("Parsing datetime pattern '{}' failed at text '{}'", s, remaining);
      debug!("{}", error);
      Err(error)
    } else {
      Ok(result)
    },
    Err(err) => {
      let error = format!("Parsing datetime pattern '{}' failed with error - {}", s, err);
      debug!("{}", error);
      Err(error)
    }
  }
}

fn validate_datetime_string(value: &String, pattern_tokens: &Vec<DateTimePatternToken>) -> Result<(), String> {
  let mut buffer = value.as_str();
  for token in pattern_tokens {
    let result = match token {
      DateTimePatternToken::Era(count) => era(buffer, *count),
      DateTimePatternToken::Year(count) => year(buffer, *count),
      DateTimePatternToken::WeekInYear => week_in_year(buffer),
      DateTimePatternToken::WeekInMonth(from_one) => week_in_month(buffer, *from_one),
      DateTimePatternToken::DayInYear => day_in_year(buffer),
      DateTimePatternToken::DayInMonth => day_in_month(buffer),
      DateTimePatternToken::Month(count) => month(buffer, *count),
      DateTimePatternToken::MonthNum(count) => month_num(buffer, *count),
      DateTimePatternToken::Text(t) => tag(t.as_str())(buffer).map(|(remaining, result)| (remaining, result.into())),
      DateTimePatternToken::DayName(count) => day_of_week_name(buffer, *count),
      DateTimePatternToken::DayOfWeek(count) => day_of_week(buffer, *count),
      DateTimePatternToken::Hour24 => hour_24(buffer),
      DateTimePatternToken::Hour24ZeroBased => hour_24_0(buffer),
      DateTimePatternToken::Hour12 => hour_12(buffer),
      DateTimePatternToken::Hour12ZeroBased => hour_12_0(buffer),
      DateTimePatternToken::Minute => minute(buffer),
      DateTimePatternToken::Second => second(buffer),
      DateTimePatternToken::Millisecond(size) => millisecond(buffer, *size),
      DateTimePatternToken::Nanosecond(_size) => digit1(buffer).map(|(remaining, result)| (remaining, result.into())),
      DateTimePatternToken::TimezoneName(size) => timezone(buffer, *size),
      DateTimePatternToken::TimezoneId(_size) => timezone_id(buffer),
      DateTimePatternToken::TimezoneOffset(size) => timezone_offset(buffer, *size),
      DateTimePatternToken::TimezoneOffsetGmt(size) => timezone_offset_gmt(buffer, *size),
      DateTimePatternToken::TimezoneOffsetX(size) => timezone_long_offset(buffer, *size),
      DateTimePatternToken::TimezoneOffsetXZZero(size) => timezone_long_offset_with_z(buffer, *size),
      DateTimePatternToken::AmPm => ampm(buffer),
      DateTimePatternToken::QuarterOfYear(count) => quarter(buffer, *count),
      DateTimePatternToken::QuarterOfYearNum(count) => quarter_num(buffer, *count),
      DateTimePatternToken::MillisecondOfDay => digit1(buffer).map(|(remaining, result)| (remaining, result.into())),
      DateTimePatternToken::NanosecondOfDay => digit1(buffer).map(|(remaining, result)| (remaining, result.into())),
    }.map_err(|err| format!("{:?}", err))?;
    buffer = result.0;
  }

  if buffer.len() > 0 {
    Err(format!("Remaining data after applying pattern {:?}", buffer))
  } else {
    Ok(())
  }
}

/// Validates the given datetime against the pattern
pub fn validate_datetime(value: &String, format: &String) -> Result<(), String> {
  match parse_pattern(format.as_str()) {
    Ok(pattern_tokens) => validate_datetime_string(value, &pattern_tokens),
    Err(err) => Err(format!("Error parsing '{}': {:?}", value, err))
  }
}

/// Converts the date time pattern tokens to a chrono formatted string
pub fn to_chrono_pattern(tokens: &Vec<DateTimePatternToken>) -> String {
  let mut buffer = String::new();

  for token in tokens {
    buffer.push_str(match token {
      DateTimePatternToken::Era(_count) => "AD".into(),
      DateTimePatternToken::Year(d) => if *d == 2 { "%y".into() } else { "%Y".into() },
      DateTimePatternToken::WeekInYear => "%U".into(),
      DateTimePatternToken::WeekInMonth(_) => {
        log::warn!("Chono does not support week in month");
        "".into()
      },
      DateTimePatternToken::DayInYear => "%j".into(),
      DateTimePatternToken::DayInMonth => "%d".into(),
      DateTimePatternToken::Month(d) => if *d <= 2 { "%m".into() } else if *d > 3 { "%B".into() } else { "%b".into() },
      DateTimePatternToken::MonthNum(_d) => "%m".into(),
      DateTimePatternToken::Text(t) => t.replace("%", "%%").to_owned(),
      DateTimePatternToken::DayName(d) => if *d > 3 { "%A".into() } else { "%a".into() },
      DateTimePatternToken::DayOfWeek(_d) => "%u".into(),
      DateTimePatternToken::Hour24 => "%H".into(),
      DateTimePatternToken::Hour24ZeroBased => "%H".into(),
      DateTimePatternToken::Hour12 => "%I".into(),
      DateTimePatternToken::Hour12ZeroBased => "%I".into(),
      DateTimePatternToken::Minute => "%M".into(),
      DateTimePatternToken::Second => "%S".into(),
      DateTimePatternToken::Millisecond(d) => format!("%{}f", *d),
      DateTimePatternToken::Nanosecond(_d) => "%f".into(),
      DateTimePatternToken::TimezoneName(_d) => "%Z".into(),
      DateTimePatternToken::TimezoneId(_d) => "%Z".into(),
      DateTimePatternToken::TimezoneOffset(_d) => "%z".into(),
      DateTimePatternToken::TimezoneOffsetX(_d) => "%:z".into(),
      DateTimePatternToken::TimezoneOffsetXZZero(_d) => "%:z".into(),
      DateTimePatternToken::AmPm => "%p".into(),
      _ => {
        log::warn!("Chono does not support {:?}", token);
        "".into()
      }
    }.as_str());
  }

  buffer
}

/// Generates a date/time string from the current system clock using the provided format string
pub fn generate_string(format: &String) -> Result<String, String> {
  match parse_pattern(format.as_str()) {
    Ok(pattern_tokens) => Ok(Local::now().format(
      to_chrono_pattern(&pattern_tokens).as_str()).to_string()),
    Err(err) => Err(format!("Error parsing '{}': {:?}", format, err))
  }
}

fn validate_tz_abbreviation(tz: &str) -> bool {
  ZONES_ABBR.contains_key(tz)
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;

  use super::*;

  #[test]
  fn parse_date_and_time() {
    expect!(validate_datetime(&"2001-01-02".into(), &"yyyy-MM-dd".into())).to(be_ok());
    expect!(validate_datetime(&"2001-01-02 12:33:45".into(), &"yyyy-MM-dd HH:mm:ss".into())).to(be_ok());

    expect!(validate_datetime(&"2001-13-02".into(), &"yyyy-MM-dd".into())).to(be_err());
    expect!(validate_datetime(&"2001-01-02 25:33:45".into(), &"yyyy-MM-dd HH:mm:ss".into())).to(be_err());

    expect!(validate_datetime(&"2001.07.04 AD at 12:08:56 PDT".into(), &"yyyy.MM.dd G 'at' HH:mm:ss z".into())).to(be_ok());
    expect!(validate_datetime(&"Wed, Jul 4, '01".into(), &"EEE, MMM d, ''yy".into())).to(be_ok());
    expect!(validate_datetime(&"12:08 PM".into(), &"h:mm a".into())).to(be_ok());
//    "hh 'o''clock' a, zzzz"	12 o'clock PM, Pacific Daylight Time
    expect!(validate_datetime(&"0:08 PM, AEST".into(), &"K:mm a, z".into())).to(be_ok());
    expect!(validate_datetime(&"02001.July.04 AD 12:08 PM".into(), &"yyyyy.MMMMM.dd G hh:mm a".into())).to(be_ok());
    expect!(validate_datetime(&"Wed, 4 Jul 2001 12:08:56 -0700".into(), &"EEE, d MMM yyyy HH:mm:ss Z".into())).to(be_ok());
    expect!(validate_datetime(&"010704120856-0700".into(), &"yyMMddHHmmssZ".into())).to(be_ok());
    expect!(validate_datetime(&"2001-07-04T12:08:56.235-0700".into(), &"yyyy-MM-dd'T'HH:mm:ss.SSSZ".into())).to(be_ok());
    expect!(validate_datetime(&"2001-07-04T12:08:56.235Z".into(), &"yyyy-MM-dd'T'HH:mm:ss.SSSX".into())).to(be_ok());
    expect!(validate_datetime(&"2001-07-04T12:08:56.235-07:00".into(), &"yyyy-MM-dd'T'HH:mm:ss.SSSXXX".into())).to(be_ok());
    expect!(validate_datetime(&"2001-W27-3".into(), &"YYYY-'W'ww-u".into())).to(be_ok());

    expect!(validate_datetime(&"2020-01-01T10:00+01:00[Europe/Warsaw]".into(), &"yyyy-MM-dd'T'HH:mmXXX'['VV']'".into())).to(be_ok());
  }

  #[test]
  fn parse_era() {
    expect!(parse_pattern("G")).to(
      be_ok().value(vec![DateTimePatternToken::Era(1)]));
    expect!(parse_pattern("GG")).to(
      be_ok().value(vec![DateTimePatternToken::Era(2)]));
    expect!(parse_pattern("GGGGG")).to(
      be_ok().value(vec![DateTimePatternToken::Era(5)]));
    expect!(parse_pattern("GGGGGG")).to(
      be_err().value("Parsing datetime pattern \'GGGGGG\' failed with error - Parsing Failure: TooManyPatternLetters(\"Too many pattern letters for Era (\\\'G\\\'): 6\", 6)".to_string()));

    expect!(validate_datetime(&"ad".into(), &"G".into())).to(be_ok());
    expect!(validate_datetime(&"AD".into(), &"GG".into())).to(be_ok());
    expect!(validate_datetime(&"bc".into(), &"GGG".into())).to(be_ok());
    expect!(validate_datetime(&"BC".into(), &"G".into())).to(be_ok());
    expect!(validate_datetime(&"BX".into(), &"G".into())).to(be_err());
  }

  #[test]
  fn parse_ampm() {
    expect!(parse_pattern("a")).to(
      be_ok().value(vec![DateTimePatternToken::AmPm]));
    expect!(parse_pattern("aa")).to(be_err());
    expect!(parse_pattern("aaaa")).to(be_err());

    expect!(validate_datetime(&"am".into(), &"a".into())).to(be_ok());
    expect!(validate_datetime(&"AM".into(), &"a".into())).to(be_ok());
    expect!(validate_datetime(&"pm".into(), &"a".into())).to(be_ok());
    expect!(validate_datetime(&"PM".into(), &"a".into())).to(be_ok());
    expect!(validate_datetime(&"PX".into(), &"a".into())).to(be_err());
  }

  #[test]
  fn parse_year() {
    expect!(parse_pattern("y")).to(
      be_ok().value(vec![DateTimePatternToken::Year(1)]));
    expect!(parse_pattern("u")).to(
      be_ok().value(vec![DateTimePatternToken::Year(1)]));
    expect!(parse_pattern("yy")).to(
      be_ok().value(vec![DateTimePatternToken::Year(2)]));
    expect!(parse_pattern("yyyy")).to(
      be_ok().value(vec![DateTimePatternToken::Year(4)]));
    expect!(parse_pattern("YYyy")).to(
      be_ok().value(vec![DateTimePatternToken::Year(2), DateTimePatternToken::Year(2)]));

    expect!(validate_datetime(&"2000".into(), &"yyyy".into())).to(be_ok());
    expect!(validate_datetime(&"200000".into(), &"yyyyyy".into())).to(be_ok());
    expect!(validate_datetime(&"20".into(), &"yy".into())).to(be_ok());
    expect!(validate_datetime(&"2000".into(), &"YYYY".into())).to(be_ok());
    expect!(validate_datetime(&"20".into(), &"YY".into())).to(be_ok());
    expect!(validate_datetime(&"20".into(), &"yyyy".into())).to(be_ok());
    expect!(validate_datetime(&"".into(), &"yyyy".into())).to(be_err());
  }

  #[test]
  fn parse_month() {
    expect!(parse_pattern("M")).to(
      be_ok().value(vec![DateTimePatternToken::Month(1)]));
    expect!(parse_pattern("MM")).to(
      be_ok().value(vec![DateTimePatternToken::Month(2)]));
    expect!(parse_pattern("LLL")).to(
      be_ok().value(vec![DateTimePatternToken::MonthNum(3)]));
    expect!(parse_pattern("MMMMMM")).to(be_err());

    expect!(validate_datetime(&"jan".into(), &"M".into())).to(be_err());
    expect!(validate_datetime(&"jan".into(), &"MMM".into())).to(be_ok());
    expect!(validate_datetime(&"october".into(), &"MMM".into())).to(be_err());
    expect!(validate_datetime(&"December".into(), &"MMMM".into())).to(be_ok());
    expect!(validate_datetime(&"December".into(), &"L".into())).to(be_err());
    expect!(validate_datetime(&"01".into(), &"L".into())).to(be_ok());
    expect!(validate_datetime(&"10".into(), &"MM".into())).to(be_ok());
    expect!(validate_datetime(&"100".into(), &"MM".into())).to(be_err());
    expect!(validate_datetime(&"100".into(), &"LL".into())).to(be_err());
    expect!(validate_datetime(&"13".into(), &"MM".into())).to(be_err());
    expect!(validate_datetime(&"31".into(), &"MM".into())).to(be_err());
    expect!(validate_datetime(&"00".into(), &"MM".into())).to(be_err());
    expect!(validate_datetime(&"".into(), &"MMM".into())).to(be_err());
  }

  #[test]
  fn parse_text() {
    expect!(parse_pattern("'ello'")).to(
      be_ok().value(vec![DateTimePatternToken::Text("ello".chars().collect())]));
    expect!(parse_pattern("'dd-MM-yyyy'")).to(
      be_ok().value(vec![DateTimePatternToken::Text("dd-MM-yyyy".chars().collect())]));
    expect!(parse_pattern("''")).to(
      be_ok().value(vec![DateTimePatternToken::Text("'".chars().collect())]));
    expect!(parse_pattern("'dd-''MM''-yyyy'")).to(
      be_ok().value(vec![DateTimePatternToken::Text("dd-'MM'-yyyy".chars().collect())]));

    expect!(validate_datetime(&"ello".into(), &"'ello'".into())).to(be_ok());
    expect!(validate_datetime(&"elo".into(), &"'ello'".into())).to(be_err());
    expect!(validate_datetime(&"dd-MM-yyyy".into(), &"'dd-MM-yyyy'".into())).to(be_ok());
  }

  #[test]
  fn parse_week_number() {
    expect!(parse_pattern("wW")).to(
      be_ok().value(vec![DateTimePatternToken::WeekInYear, DateTimePatternToken::WeekInMonth(true)]));
    expect!(parse_pattern("www")).to(be_err());
    expect!(parse_pattern("WW")).to(
      be_ok().value(vec![DateTimePatternToken::WeekInMonth(true)]));
    expect!(parse_pattern("F")).to(
      be_ok().value(vec![DateTimePatternToken::WeekInMonth(false)]));

    expect!(validate_datetime(&"12".into(), &"w".into())).to(be_ok());
    expect!(validate_datetime(&"3".into(), &"WW".into())).to(be_ok());
    expect!(validate_datetime(&"57".into(), &"ww".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"W".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"F".into())).to(be_ok());
  }

  #[test]
  fn parse_day_number() {
    expect!(parse_pattern("dD")).to(
      be_ok().value(vec![DateTimePatternToken::DayInMonth, DateTimePatternToken::DayInYear]));
    expect!(parse_pattern("dd")).to(
      be_ok().value(vec![DateTimePatternToken::DayInMonth]));
    expect!(parse_pattern("DDD")).to(
      be_ok().value(vec![DateTimePatternToken::DayInYear]));
    expect!(parse_pattern("ddd")).to(be_err());

    expect!(validate_datetime(&"12".into(), &"d".into())).to(be_ok());
    expect!(validate_datetime(&"03".into(), &"DD".into())).to(be_ok());
    expect!(validate_datetime(&"32".into(), &"dd".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"D".into())).to(be_err());
    expect!(validate_datetime(&"357".into(), &"D".into())).to(be_err());
  }

  #[test]
  fn parse_day_of_week() {
    expect!(parse_pattern("c")).to(
      be_ok().value(vec![DateTimePatternToken::DayOfWeek(1)]));
    expect!(parse_pattern("EE")).to(
      be_ok().value(vec![DateTimePatternToken::DayName(2)]));
    expect!(parse_pattern("ee")).to(
      be_ok().value(vec![DateTimePatternToken::DayOfWeek(2)]));

    expect!(validate_datetime(&"7".into(), &"c".into())).to(be_ok());
    expect!(validate_datetime(&"Tue".into(), &"EEE".into())).to(be_ok());
    expect!(validate_datetime(&"Tuesday".into(), &"EEEE".into())).to(be_ok());
    expect!(validate_datetime(&"3".into(), &"E".into())).to(be_err());
    expect!(validate_datetime(&"3".into(), &"e".into())).to(be_ok());
    expect!(validate_datetime(&"32".into(), &"ee".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"c".into())).to(be_err());
  }

  #[test]
  fn parse_hour() {
    expect!(parse_pattern("k")).to(
      be_ok().value(vec![DateTimePatternToken::Hour24]));
    expect!(parse_pattern("KK")).to(
      be_ok().value(vec![DateTimePatternToken::Hour12ZeroBased]));
    expect!(parse_pattern("hh")).to(
      be_ok().value(vec![DateTimePatternToken::Hour12]));
    expect!(parse_pattern("HH")).to(
      be_ok().value(vec![DateTimePatternToken::Hour24ZeroBased]));
    expect!(parse_pattern("HHHH")).to(be_err());

    expect!(validate_datetime(&"11".into(), &"k".into())).to(be_ok());
    expect!(validate_datetime(&"11".into(), &"KK".into())).to(be_ok());
    expect!(validate_datetime(&"11".into(), &"hh".into())).to(be_ok());
    expect!(validate_datetime(&"11".into(), &"H".into())).to(be_ok());

    expect!(validate_datetime(&"25".into(), &"kk".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"k".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"KK".into())).to(be_ok());
    expect!(validate_datetime(&"12".into(), &"KK".into())).to(be_err());
    expect!(validate_datetime(&"12".into(), &"h".into())).to(be_ok());
    expect!(validate_datetime(&"0".into(), &"hh".into())).to(be_err());
    expect!(validate_datetime(&"0".into(), &"H".into())).to(be_ok());
    expect!(validate_datetime(&"23".into(), &"H".into())).to(be_ok());
    expect!(validate_datetime(&"24".into(), &"HH".into())).to(be_err());
  }

  #[test]
  fn parse_minute_and_second() {
    expect!(parse_pattern("m")).to(
      be_ok().value(vec![DateTimePatternToken::Minute]));
    expect!(parse_pattern("s")).to(
      be_ok().value(vec![DateTimePatternToken::Second]));
    expect!(parse_pattern("SSS")).to(
      be_ok().value(vec![DateTimePatternToken::Millisecond(3)]));
    expect!(parse_pattern("A")).to(
      be_ok().value(vec![DateTimePatternToken::MillisecondOfDay]));
    expect!(parse_pattern("n")).to(
      be_ok().value(vec![DateTimePatternToken::Nanosecond(1)]));
    expect!(parse_pattern("N")).to(
      be_ok().value(vec![DateTimePatternToken::NanosecondOfDay]));

    expect!(validate_datetime(&"12".into(), &"m".into())).to(be_ok());
    expect!(validate_datetime(&"03".into(), &"ss".into())).to(be_ok());
    expect!(validate_datetime(&"030".into(), &"SSS".into())).to(be_ok());
    expect!(validate_datetime(&"35392790".into(), &"A".into())).to(be_ok());
    expect!(validate_datetime(&"35392790".into(), &"n".into())).to(be_ok());
    expect!(validate_datetime(&"60".into(), &"m".into())).to(be_err());
    expect!(validate_datetime(&"61".into(), &"s".into())).to(be_err());
    expect!(validate_datetime(&"1000".into(), &"SS".into())).to(be_err());
  }

  #[test]
  fn parse_timezone() {
    expect!(parse_pattern("x")).to(
      be_ok().value(vec![DateTimePatternToken::TimezoneOffsetX(1)]));
    expect!(parse_pattern("Z")).to(
      be_ok().value(vec![DateTimePatternToken::TimezoneOffset(1)]));
    expect!(parse_pattern("XXX")).to(
      be_ok().value(vec![DateTimePatternToken::TimezoneOffsetXZZero(3)]));
    expect!(parse_pattern("OOOO")).to(
      be_ok().value(vec![DateTimePatternToken::TimezoneOffsetGmt(4)]));

    expect!(validate_datetime(&"-0700".into(), &"Z".into())).to(be_ok());
    expect!(validate_datetime(&"1100".into(), &"ZZZZ".into())).to(be_err());
    expect!(validate_datetime(&"GMT+10:00".into(), &"ZZZZ".into())).to(be_ok());
    expect!(validate_datetime(&"+1030".into(), &"Z".into())).to(be_ok());
    expect!(validate_datetime(&"-2400".into(), &"Z".into())).to(be_err());
    expect!(validate_datetime(&"2361".into(), &"Z".into())).to(be_err());
    expect!(validate_datetime(&"Z".into(), &"Z".into())).to(be_err());
    expect!(validate_datetime(&"GMT".into(), &"ZZZZ".into())).to(be_ok());
    expect!(validate_datetime(&"+10:00".into(), &"ZZZZZ".into())).to(be_ok());

    expect!(validate_datetime(&"Z".into(), &"X".into())).to(be_ok());
    expect!(validate_datetime(&"Z".into(), &"x".into())).to(be_err());
    expect!(validate_datetime(&"-0730".into(), &"X".into())).to(be_ok());
    expect!(validate_datetime(&"+08".into(), &"X".into())).to(be_ok());
    expect!(validate_datetime(&"-0730".into(), &"x".into())).to(be_ok());
    expect!(validate_datetime(&"+0800".into(), &"x".into())).to(be_ok());
    expect!(validate_datetime(&"-0730".into(), &"XX".into())).to(be_ok());
    expect!(validate_datetime(&"+0800".into(), &"xx".into())).to(be_ok());
    expect!(validate_datetime(&"-07:30".into(), &"XXX".into())).to(be_ok());
    expect!(validate_datetime(&"+08:00".into(), &"xxx".into())).to(be_ok());
    expect!(validate_datetime(&"-0730".into(), &"XXXX".into())).to(be_ok());
    expect!(validate_datetime(&"+0800".into(), &"xxxx".into())).to(be_ok());
    expect!(validate_datetime(&"-073000".into(), &"XXXX".into())).to(be_ok());
    expect!(validate_datetime(&"+080000".into(), &"xxxx".into())).to(be_ok());
    expect!(validate_datetime(&"-07:30:00".into(), &"XXXXX".into())).to(be_ok());
    expect!(validate_datetime(&"+08:00:00".into(), &"xxxxx".into())).to(be_ok());

    expect!(validate_datetime(&"1100".into(), &"XX".into())).to(be_err());
    expect!(validate_datetime(&"1100".into(), &"xx".into())).to(be_err());
    expect!(validate_datetime(&"+10".into(), &"XX".into())).to(be_err());
    expect!(validate_datetime(&"+10".into(), &"xx".into())).to(be_err());
    expect!(validate_datetime(&"-0730".into(), &"XXX".into())).to(be_err());
    expect!(validate_datetime(&"+0800".into(), &"xxx".into())).to(be_err());
    expect!(validate_datetime(&"-07:30".into(), &"XXXX".into())).to(be_err());
    expect!(validate_datetime(&"+08:00".into(), &"xxxx".into())).to(be_err());
    expect!(validate_datetime(&"-073000".into(), &"XXXXX".into())).to(be_err());
    expect!(validate_datetime(&"+080000".into(), &"xxxxx".into())).to(be_err());

    expect!(validate_datetime(&"GMT-7".into(), &"O".into())).to(be_ok());
    expect!(validate_datetime(&"UTC+10".into(), &"O".into())).to(be_ok());
    expect!(validate_datetime(&"UTC+9:30".into(), &"O".into())).to(be_ok());
    expect!(validate_datetime(&"GMT+08:00".into(), &"OOOO".into())).to(be_ok());
    expect!(validate_datetime(&"GMT+08".into(), &"OOOO".into())).to(be_err());

    // expect!(validate_datetime(&"AEST".into(), &"z".into())).to(be_ok());
    // expect!(validate_datetime(&"BST".into(), &"z".into())).to(be_ok());
    // expect!(validate_datetime(&"UTC".into(), &"z".into())).to(be_ok());
    // expect!(validate_datetime(&"aest".into(), &"z".into())).to(be_err());
    // expect!(validate_datetime(&"AEST".into(), &"zzzz".into())).to(be_err());
  }

  #[test]
  fn to_chrono_pattern_test() {
    expect!(to_chrono_pattern(&parse_pattern("yyyy-MM-dd").unwrap())).to(be_equal_to("%Y-%m-%d"));
    expect!(to_chrono_pattern(&parse_pattern("yyyy-MM-dd HH:mm:ss").unwrap())).to(be_equal_to("%Y-%m-%d %H:%M:%S"));
    expect!(to_chrono_pattern(&parse_pattern("EEE, MMM d, ''yy").unwrap())).to(be_equal_to("%a, %b %d, \'%y"));
    expect!(to_chrono_pattern(&parse_pattern("h:mm a").unwrap())).to(be_equal_to("%I:%M %p"));
    expect!(to_chrono_pattern(&parse_pattern("hh 'o''clock' a, z").unwrap())).to(be_equal_to("%I o'clock %p, %Z"));
    expect!(to_chrono_pattern(&parse_pattern("yyyyy.MMMMM.dd GGG hh:mm a").unwrap())).to(be_equal_to("%Y.%B.%d AD %I:%M %p"));
    expect!(to_chrono_pattern(&parse_pattern("EEE, d MMM yyyy HH:mm:ss Z").unwrap())).to(be_equal_to("%a, %d %b %Y %H:%M:%S %z"));
    expect!(to_chrono_pattern(&parse_pattern("yyMMddHHmmssZ").unwrap())).to(be_equal_to("%y%m%d%H%M%S%z"));
    expect!(to_chrono_pattern(&parse_pattern("yyyy-MM-dd'T'HH:mm:ss.SSSZ").unwrap())).to(be_equal_to("%Y-%m-%dT%H:%M:%S.%3f%z"));
    expect!(to_chrono_pattern(&parse_pattern("yyyy-MM-dd'T'HH:mm:ss.SSSXXX").unwrap())).to(be_equal_to("%Y-%m-%dT%H:%M:%S.%3f%:z"));
    expect!(to_chrono_pattern(&parse_pattern("YYYY-'W'ww-e").unwrap())).to(be_equal_to("%Y-W%U-%u"));
  }

  #[test]
  fn parse_quarter() {
    expect!(parse_pattern("Q")).to(
      be_ok().value(vec![DateTimePatternToken::QuarterOfYear(1)]));
    expect!(parse_pattern("QQ")).to(
      be_ok().value(vec![DateTimePatternToken::QuarterOfYear(2)]));
    expect!(parse_pattern("QQQ")).to(
      be_ok().value(vec![DateTimePatternToken::QuarterOfYear(3)]));
    expect!(parse_pattern("QQQQQQ")).to(be_err());
    expect!(parse_pattern("q")).to(
      be_ok().value(vec![DateTimePatternToken::QuarterOfYearNum(1)]));
    expect!(parse_pattern("qqq")).to(be_err());

    expect!(validate_datetime(&"2".into(), &"Q".into())).to(be_ok());
    expect!(validate_datetime(&"2".into(), &"q".into())).to(be_ok());
    expect!(validate_datetime(&"02".into(), &"QQ".into())).to(be_ok());
    expect!(validate_datetime(&"02".into(), &"qq".into())).to(be_ok());
    expect!(validate_datetime(&"Q2".into(), &"QQ".into())).to(be_err());
    expect!(validate_datetime(&"Q2".into(), &"QQQ".into())).to(be_ok());
    expect!(validate_datetime(&"Q2".into(), &"qq".into())).to(be_err());
    expect!(validate_datetime(&"2nd quarter".into(), &"QQQQ".into())).to(be_ok());
    expect!(validate_datetime(&"5th quarter".into(), &"QQQQ".into())).to(be_err());
  }

  #[test]
  fn timezone_abbreviations() {
    expect!(validate_tz_abbreviation("AEST")).to(be_true());
    expect!(validate_tz_abbreviation("AEDT")).to(be_true());
    expect!(validate_tz_abbreviation("XXX")).to(be_false());
  }
}
