//! Functions for dealing with path expressions

use std::iter::Peekable;

/// Struct to store path token
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathToken {
  /// Root token $
  Root,
  /// named field token
  Field(String),
  /// integer index token
  Index(usize),
  /// * token
  Star,
  /// * index token
  StarIndex
}

fn peek<I>(chars: &mut Peekable<I>) -> Option<(usize, char)> where I: Iterator<Item = (usize, char)> {
  chars.peek().map(|tup| (tup.0.clone(), tup.1.clone()))
}

fn is_identifier_char(ch: char) -> bool {
  ch.is_alphabetic() || ch.is_numeric() || ch == '_' || ch == '-' || ch == ':' || ch == '#' || ch == '@'
}

// identifier -> a-zA-Z0-9+
fn identifier<I>(ch: char, chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut id = String::new();
  id.push(ch);
  let mut next_char = peek(chars);
  while next_char.is_some() {
    let ch = next_char.unwrap();
    if is_identifier_char(ch.1) {
      chars.next();
      id.push(ch.1);
    } else if ch.1 == '.' || ch.1 == '\'' || ch.1 == '[' {
      break;
    } else {
      return Err(format!("\"{}\" is not allowed in an identifier in path expression \"{}\" at index {}",
                         ch.1, path, ch.0));
    }
    next_char = peek(chars);
  }
  tokens.push(PathToken::Field(id));
  Ok(())
}

// path_identifier -> identifier | *
fn path_identifier<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str, index: usize) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  match chars.next() {
    Some(ch) => match ch.1 {
      '*' => {
        tokens.push(PathToken::Star);
        Ok(())
      },
      c if is_identifier_char(c) => {
        identifier(c, chars, tokens, path)?;
        Ok(())
      },
      _ => Err(format!("Expected either a \"*\" or path identifier in path expression \"{}\" at index {}",
                       path, ch.0))
    },
    None => Err(format!("Expected a path after \".\" in path expression \"{}\" at index {}",
                        path, index))
  }
}

// string_path -> [^']+
fn string_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str, index: usize) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut id = String::new();
  let mut next_char = peek(chars);
  if next_char.is_some() {
    chars.next();
    let mut ch = next_char.unwrap();
    next_char = peek(chars);
    while ch.1 != '\'' && next_char.is_some() {
      id.push(ch.1);
      chars.next();
      ch = next_char.unwrap();
      next_char = peek(chars);
    }
    if ch.1 == '\'' {
      if id.is_empty() {
        Err(format!("Empty strings are not allowed in path expression \"{}\" at index {}", path, ch.0))
      } else {
        tokens.push(PathToken::Field(id));
        Ok(())
      }
    } else {
      Err(format!("Unterminated string in path expression \"{}\" at index {}", path, ch.0))
    }
  } else {
    Err(format!("Unterminated string in path expression \"{}\" at index {}", path, index))
  }
}

// index_path -> [0-9]+
fn index_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut id = String::new();
  let mut next_char = chars.next();
  id.push(next_char.unwrap().1);
  next_char = peek(chars);
  while next_char.is_some() {
    let ch = next_char.unwrap();
    if ch.1.is_numeric() {
      id.push(ch.1);
      chars.next();
    } else {
      break;
    }
    next_char = peek(chars);
  }

  if let Some(ch) = next_char {
    if ch.1 != ']' {
      return Err(format!("Indexes can only consist of numbers or a \"*\", found \"{}\" instead in path expression \"{}\" at index {}",
                         ch.1, path, ch.0))
    }
  }

  tokens.push(PathToken::Index(id.parse().unwrap()));
  Ok(())
}

// bracket_path -> (string_path | index | *) ]
fn bracket_path<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str, index: usize) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut ch = peek(chars);
  match ch {
    Some(c) => {
      if c.1 == '\'' {
        chars.next();
        string_path(chars, tokens, path, c.0)?
      } else if c.1.is_numeric() {
        index_path(chars, tokens, path)?
      } else if c.1 == '*' {
        chars.next();
        tokens.push(PathToken::StarIndex);
      } else if c.1 == ']' {
        return Err(format!("Empty bracket expressions are not allowed in path expression \"{}\" at index {}",
                           path, c.0));
      } else {
        return Err(format!("Indexes can only consist of numbers or a \"*\", found \"{}\" instead in path expression \"{}\" at index {}",
                           c.1, path, c.0));
      };
      ch = peek(chars);
      match ch {
        Some(c) => if c.1 != ']' {
          Err(format!("Unterminated brackets, found \"{}\" instead of \"]\" in path expression \"{}\" at index {}",
                      c.1, path, c.0))
        } else {
          chars.next();
          Ok(())
        },
        None => Err(format!("Unterminated brackets in path expression \"{}\" at index {}",
                            path, path.len() - 1))
      }
    },
    None => Err(format!("Expected a \"'\" (single qoute) or a digit in path expression \"{}\" after index {}",
                        path, index))
  }
}

// path_exp -> (dot-path | bracket-path)*
fn path_exp<I>(chars: &mut Peekable<I>, tokens: &mut Vec<PathToken>, path: &str) -> Result<(), String>
  where I: Iterator<Item=(usize, char)> {
  let mut next_char = chars.next();
  while next_char.is_some() {
    let ch = next_char.unwrap();
    match ch.1 {
      '.' => path_identifier(chars, tokens, path, ch.0)?,
      '[' => bracket_path(chars, tokens, path, ch.0)?,
      _ => return Err(format!("Expected a \".\" or \"[\" instead of \"{}\" in path expression \"{}\" at index {}",
                              ch.1, path, ch.0))
    }
    next_char = chars.next();
  }
  Ok(())
}

pub fn parse_path_exp(path: &str) -> Result<Vec<PathToken>, String> {
  let mut tokens = vec![];

  // parse_path_exp -> $ path_exp | empty
  let mut chars = path.chars().enumerate().peekable();
  match chars.next() {
    Some(ch) => {
      match ch.1 {
        '$' => {
          tokens.push(PathToken::Root);
          path_exp(&mut chars, &mut tokens, path)?;
          Ok(tokens)
        }
        c if c.is_alphabetic() || c.is_numeric() => {
          tokens.push(PathToken::Root);
          identifier(c, &mut chars, &mut tokens, path)?;
          path_exp(&mut chars, &mut tokens, path)?;
          Ok(tokens)
        }
        _ => Err(format!("Path expression \"{}\" does not start with a root marker \"$\"", path))
      }
    }
    None => Ok(tokens)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use expectest::prelude::*;
  use expectest::expect;

  #[test]
  fn parse_path_exp_handles_empty_string() {
    expect!(parse_path_exp("")).to(be_ok().value(vec![]));
  }

  #[test]
  fn parse_path_exp_handles_root() {
    expect!(parse_path_exp("$")).to(be_ok().value(vec![PathToken::Root]));
  }

  #[test]
  fn parse_path_exp_handles_missing_root() {
    expect!(parse_path_exp("adsjhaskjdh"))
      .to(be_ok().value(vec![PathToken::Root, PathToken::Field("adsjhaskjdh".to_string())]));
  }

  #[test]
  fn parse_path_exp_handles_missing_path() {
    expect!(parse_path_exp("$adsjhaskjdh")).to(
      be_err().value("Expected a \".\" or \"[\" instead of \"a\" in path expression \"$adsjhaskjdh\" at index 1".to_string()));
  }

  #[test]
  fn parse_path_exp_handles_missing_path_name() {
    expect!(parse_path_exp("$.")).to(
      be_err().value("Expected a path after \".\" in path expression \"$.\" at index 1".to_string()));
    expect!(parse_path_exp("$.a.b.c.")).to(
      be_err().value("Expected a path after \".\" in path expression \"$.a.b.c.\" at index 7".to_string()));
  }

  #[test]
  fn parse_path_exp_handles_invalid_identifiers() {
    expect!(parse_path_exp("$.abc!")).to(
      be_err().value("\"!\" is not allowed in an identifier in path expression \"$.abc!\" at index 5".to_string()));
    expect!(parse_path_exp("$.a.b.c.}")).to(
      be_err().value("Expected either a \"*\" or path identifier in path expression \"$.a.b.c.}\" at index 8".to_string()));
  }

  #[test]
  fn parse_path_exp_with_simple_identifiers() {
    expect!(parse_path_exp("$.a")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string())]));
    expect!(parse_path_exp("$.a.b.c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Field("b".to_string()),
                         PathToken::Field("c".to_string())]));
    expect!(parse_path_exp("a.b.c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Field("b".to_string()),
                         PathToken::Field("c".to_string())]));
  }

  #[test]
  fn parse_path_exp_handles_underscores_and_dashes() {
    expect!(parse_path_exp("$.user_id.user-id")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("user_id".to_string()),
                         PathToken::Field("user-id".to_string())])
    );
    expect!(parse_path_exp("$._id")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("_id".to_string())])
    );
    expect!(parse_path_exp("$.id:test")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("id:test".to_string())])
    );
  }

  #[test]
  fn parse_path_exp_handles_xml_names() {
    expect!(parse_path_exp("$.foo.@val")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("foo".to_string()),
                         PathToken::Field("@val".to_string())])
    );
    expect!(parse_path_exp("$.foo.#text")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("foo".to_string()),
                         PathToken::Field("#text".to_string())])
    );
    expect!(parse_path_exp("$.urn:ns:foo.urn:ns:something.#text")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("urn:ns:foo".to_string()),
                         PathToken::Field("urn:ns:something".to_string()), PathToken::Field("#text".to_string())])
    );
  }

  #[test]
  fn parse_path_exp_with_star_instead_of_identifiers() {
    expect!(parse_path_exp("$.*")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Star]));
    expect!(parse_path_exp("$.a.*.c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Star,
                         PathToken::Field("c".to_string())]));
  }

  #[test]
  fn parse_path_exp_with_bracket_notation() {
    expect!(parse_path_exp("$['val1']")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("val1".to_string())]));
    expect!(parse_path_exp("$.a['val@1.'].c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Field("val@1.".to_string()),
                         PathToken::Field("c".to_string())]));
    expect!(parse_path_exp("$.a[1].c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::Index(1),
                         PathToken::Field("c".to_string())]));
    expect!(parse_path_exp("$.a[*].c")).to(
      be_ok().value(vec![PathToken::Root, PathToken::Field("a".to_string()), PathToken::StarIndex,
                         PathToken::Field("c".to_string())]));
  }

  #[test]
  fn parse_path_exp_with_invalid_bracket_notation() {
    expect!(parse_path_exp("$[")).to(
      be_err().value("Expected a \"'\" (single qoute) or a digit in path expression \"$[\" after index 1".to_string()));
    expect!(parse_path_exp("$['")).to(
      be_err().value("Unterminated string in path expression \"$['\" at index 2".to_string()));
    expect!(parse_path_exp("$['Unterminated string")).to(
      be_err().value("Unterminated string in path expression \"$['Unterminated string\" at index 21".to_string()));
    expect!(parse_path_exp("$['']")).to(
      be_err().value("Empty strings are not allowed in path expression \"$['']\" at index 3".to_string()));
    expect!(parse_path_exp("$['test'.b.c")).to(
      be_err().value("Unterminated brackets, found \".\" instead of \"]\" in path expression \"$['test'.b.c\" at index 8".to_string()));
    expect!(parse_path_exp("$['test'")).to(
      be_err().value("Unterminated brackets in path expression \"$['test'\" at index 7".to_string()));
    expect!(parse_path_exp("$['test']b.c")).to(
      be_err().value("Expected a \".\" or \"[\" instead of \"b\" in path expression \"$[\'test\']b.c\" at index 9".to_string()));
  }

  #[test]
  fn parse_path_exp_with_invalid_bracket_index_notation() {
    expect!(parse_path_exp("$[dhghh]")).to(
      be_err().value("Indexes can only consist of numbers or a \"*\", found \"d\" instead in path expression \"$[dhghh]\" at index 2".to_string()));
    expect!(parse_path_exp("$[12abc]")).to(
      be_err().value("Indexes can only consist of numbers or a \"*\", found \"a\" instead in path expression \"$[12abc]\" at index 4".to_string()));
    expect!(parse_path_exp("$[]")).to(
      be_err().value("Empty bracket expressions are not allowed in path expression \"$[]\" at index 2".to_string()));
    expect!(parse_path_exp("$[-1]")).to(
      be_err().value("Indexes can only consist of numbers or a \"*\", found \"-\" instead in path expression \"$[-1]\" at index 2".to_string()));
  }
}
