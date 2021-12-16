//! Functions for dealing with path expressions

use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Write};
use std::hash::{Hash, Hasher};
use std::iter::Peekable;

use anyhow::anyhow;
use lazy_static::lazy_static;
use log::trace;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

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

fn matches_token(path_fragment: &str, path_token: &PathToken) -> usize {
  match path_token {
    PathToken::Root if path_fragment == "$" => 2,
    PathToken::Field(name) if path_fragment == name => 2,
    PathToken::Index(index) => match path_fragment.parse::<usize>() {
      Ok(i) if *index == i => 2,
      _ => 0
    },
    PathToken::StarIndex => match path_fragment.parse::<usize>() {
      Ok(_) => 1,
      _ => 0
    },
    PathToken::Star => 1,
    _ => 0
  }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct DocPath {
  path_tokens: Vec<PathToken>,
  expr: String,
}

impl DocPath {
  /// Construct a new document path from the provided string path
  pub fn new(expr: impl Into<String>) -> anyhow::Result<Self> {
    let expr = expr.into();
    let path_tokens = parse_path_exp(&expr)
      .map_err(|e| anyhow!(e))?;
    Ok(Self {
      path_tokens,
      expr,
    })
  }

  /// Infallible construction for when the expression is statically known,
  /// intended for unit tests.
  ///
  /// Invalid expressions will still cause panics.
  pub fn new_unwrap(expr: &'static str) -> Self {
    Self::new(expr).unwrap()
  }

  /// Construct a new DocPath with an empty expression.
  ///
  /// Warning: do not call any of the `push_*` methods on this DocPath,
  /// as that would create an expression with invalid syntax
  /// (because it would be missing the Root token).
  pub fn empty() -> Self {
    Self {
      path_tokens: vec![],
      expr: "".into(),
    }
  }

  /// Construct a new DocPath with the Root token.
  pub fn root() -> Self {
    Self {
      path_tokens: vec![PathToken::Root],
      expr: "$".into(),
    }
  }

  /// Return the list of tokens that comprise this path.
  pub fn tokens(&self) -> &Vec<PathToken> {
    &self.path_tokens
  }

  /// Return the length, in parsed tokens.
  pub fn len(&self) -> usize {
    self.path_tokens.len()
  }

  /// Extract the string contents of the first Field token.
  /// For use with Header and Query DocPaths.
  pub fn first_field(&self) -> Option<&str> {
    for token in self.path_tokens.iter() {
      if let PathToken::Field(ref field) = token {
        return Some(field);
      }
    }
    return None;
  }

  /// If this path is the root path (it has only one element, the root token `$`).
  pub fn is_root(&self) -> bool {
    &self.path_tokens == &[PathToken::Root]
  }

  /// The path is a wildcard path if it ends in a star (`*`)
  pub fn is_wildcard(&self) -> bool {
    self.path_tokens.last() == Some(&PathToken::Star)
  }

  /// Calculates the path weight for this path expression and a given path.
  /// Returns a tuple of the calculated weight and the number of path tokens matched.
  pub fn path_weight(&self, path: &[&str]) -> (usize, usize) {
    trace!("Calculating weight for path tokens '{:?}' and path '{:?}'",
           self.path_tokens, path);
    let weight = {
      if path.len() >= self.len() {
        (
          self.path_tokens.iter().zip(path.iter())
          .fold(1, |acc, (token, fragment)| acc * matches_token(fragment, token)),
          self.len()
        )
      } else {
        (0, self.len())
      }
    };
    trace!("Calculated weight {:?} for path '{}' and '{:?}'",
           weight, self, path);
    weight
  }

  /// If this path matches the given path. It will match if the calculated path weight is greater
  /// than zero (which means at least one token matched).
  pub fn matches_path(&self, path: &[&str]) -> bool {
    self.path_weight(path).0 > 0
  }

  /// If the path matches the given path (the calculated path weight is greater than zero) and
  /// both paths have the same length.
  pub fn matches_path_exactly(&self, path: &[&str]) -> bool {
     self.len() == path.len() && self.matches_path(path)
  }

  /// Creates a new path by cloning this one and pushing the name onto the end
  pub fn join(&self, field: impl Into<String>) -> Self {
    let field = field.into();
    let mut path = self.clone();
    path.push_field(field);
    path
  }

  /// Mutates this path by pushing a field value onto the end.
  pub fn push_field(&mut self, field: impl Into<String>) -> &mut Self {
    let field = field.into();
    write_obj_key_for_path(&mut self.expr, &field);
    self.path_tokens.push(PathToken::Field(field));
    self
  }

  /// Mutates this path by pushing an index value onto the end.
  pub fn push_index(&mut self, index: usize) -> &mut Self {
    self.path_tokens.push(PathToken::Index(index));
    // unwrap is safe, as write! is infallible for String
    write!(self.expr, "[{}]", index).unwrap();
    self
  }

  /// Mutates this path by pushing a star value onto the end.
  pub fn push_star(&mut self) -> &mut Self {
    self.path_tokens.push(PathToken::Star);
    self.expr.push_str(".*");
    self
  }

  /// Mutates this path by pushing a star index value onto the end.
  pub fn push_star_index(&mut self) -> &mut Self {
    self.path_tokens.push(PathToken::StarIndex);
    self.expr.push_str("[*]");
    self
  }
}

/// Format a JSON object key for use in a JSON path expression. If we were
/// more concerned about performance, we might try to come up with a scheme
/// to minimize string allocation here.
fn write_obj_key_for_path(mut out: impl Write, key: &str) {
  lazy_static! {
    // Only use "." syntax for things which are obvious identifiers.
    static ref IDENT: Regex = Regex::new(r#"^[_A-Za-z][_A-Za-z0-9]*$"#)
      .expect("could not parse IDENT regex");
    // Escape these characters when using string syntax.
    static ref ESCAPE: Regex = Regex::new(r#"\\|'"#)
      .expect("could not parse ESCAPE regex");
  }

  // unwrap is safe, as write! is infallible for String
  if IDENT.is_match(key) {
    write!(out, ".{}", key).unwrap();
  } else {
    write!(
      out,
      "['{}']",
      ESCAPE.replace_all(key, |caps: &Captures| format!(r#"\{}"#, &caps[0]))
    ).unwrap();
  }
}

#[cfg(test)]
fn obj_key_for_path(key: &str) -> String {
  let mut out = String::new();
  write_obj_key_for_path(&mut out, key);
  out
}

impl From<DocPath> for String {
  fn from(doc_path: DocPath) -> String {
    doc_path.expr
  }
}

impl From<&DocPath> for String {
  fn from(doc_path: &DocPath) -> String {
    doc_path.expr.clone()
  }
}

impl TryFrom<String> for DocPath {
  type Error = anyhow::Error;

  fn try_from(path: String) -> Result<Self, Self::Error> {
    DocPath::new(path)
  }
}

impl PartialEq for DocPath {
  fn eq(&self, other: &Self) -> bool {
    self.expr == other.expr
  }
}

impl Hash for DocPath {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.expr.hash(state);
  }
}

impl Display for DocPath {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "{}", self.expr)
  }
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
  fn matches_token_test_with_root() {
    expect!(matches_token("$", &PathToken::Root)).to(be_equal_to(2));
    expect!(matches_token("path", &PathToken::Root)).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::Root)).to(be_equal_to(0));
  }

  #[test]
  fn matches_token_test_with_field() {
    expect!(matches_token("$", &PathToken::Field("path".to_string()))).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::Field("path".to_string()))).to(be_equal_to(2));
  }

  #[test]
  fn matches_token_test_with_index() {
    expect!(matches_token("$", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("1", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("2", &PathToken::Index(2))).to(be_equal_to(2));
  }

  #[test]
  fn matches_token_test_with_index_wildcard() {
    expect!(matches_token("$", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("1", &PathToken::StarIndex)).to(be_equal_to(1));
  }

  #[test]
  fn matches_token_test_with_wildcard() {
    expect!(matches_token("$", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("path", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("*", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("1", &PathToken::Star)).to(be_equal_to(1));
  }

  #[test]
  fn docpath_empty() {
    expect!(DocPath::empty().path_tokens)
      .to(be_equal_to(DocPath::new_unwrap("").path_tokens));
  }

  #[test]
  fn docpath_root() {
    expect!(DocPath::root().path_tokens)
      .to(be_equal_to(DocPath::new_unwrap("$").path_tokens));
  }

  #[test]
  fn docpath_wildcard() {
    expect!(DocPath::new_unwrap("").is_wildcard()).to(be_equal_to(false));
    expect!(DocPath::new_unwrap("$.path").is_wildcard()).to(be_equal_to(false));
    expect!(DocPath::new_unwrap("$.*").is_wildcard()).to(be_equal_to(true));
    expect!(DocPath::new_unwrap("$.path.*").is_wildcard()).to(be_equal_to(true));
  }

  #[test]
  fn matches_path_matches_root_path_element() {
    expect!(DocPath::new_unwrap("$").path_weight(&vec!["$"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$").path_weight(&vec![]).0 > 0).to(be_false());
  }

  #[test]
  fn matches_path_matches_field_name() {
    expect!(DocPath::new_unwrap("$.name").path_weight(&vec!["$", "name"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$['name']").path_weight(&vec!["$", "name"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.name.other").path_weight(&vec!["$", "name", "other"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$['name'].other").path_weight(&vec!["$", "name", "other"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.name").path_weight(&vec!["$", "other"]).0 > 0).to(be_false());
    expect!(DocPath::new_unwrap("$.name").path_weight(&vec!["$", "name", "other"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.other").path_weight(&vec!["$", "name", "other"]).0 > 0).to(be_false());
    expect!(DocPath::new_unwrap("$.name.other").path_weight(&vec!["$", "name"]).0 > 0).to(be_false());
  }

  #[test]
  fn matches_path_matches_array_indices() {
    expect!(DocPath::new_unwrap("$[0]").path_weight(&vec!["$", "0"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.name[1]").path_weight(&vec!["$", "name", "1"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.name").path_weight(&vec!["$", "0"]).0 > 0).to(be_false());
    expect!(DocPath::new_unwrap("$.name[1]").path_weight(&vec!["$", "name", "0"]).0 > 0).to(be_false());
    expect!(DocPath::new_unwrap("$[1].name").path_weight(&vec!["$", "name", "1"]).0 > 0).to(be_false());
  }

  #[test]
  fn matches_path_matches_with_wildcard() {
    expect!(DocPath::new_unwrap("$[*]").path_weight(&vec!["$", "0"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.*").path_weight(&vec!["$", "name"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.*.name").path_weight(&vec!["$", "some", "name"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.name[*]").path_weight(&vec!["$", "name", "0"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$.name[*].name").path_weight(&vec!["$", "name", "1", "name"]).0 > 0).to(be_true());
    expect!(DocPath::new_unwrap("$[*]").path_weight(&vec!["$", "name"]).0 > 0).to(be_false());
  }

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

  #[test]
  fn obj_key_for_path_quotes_keys_when_necessary() {
    assert_eq!(obj_key_for_path("foo"), ".foo");
    assert_eq!(obj_key_for_path("_foo"), "._foo");
    assert_eq!(obj_key_for_path("["), "['[']");

    // I don't actually know how the JSON Path specification wants us to handle
    // these cases, but we need to _something_ to avoid panics or passing
    // `Result` around everywhere, so let's go with JavaScript string escape
    // syntax.
    assert_eq!(obj_key_for_path(r#"''"#), r#"['\'\'']"#);
    assert_eq!(obj_key_for_path(r#"a'"#), r#"['a\'']"#);
    assert_eq!(obj_key_for_path(r#"\"#), r#"['\\']"#);
  }
}
