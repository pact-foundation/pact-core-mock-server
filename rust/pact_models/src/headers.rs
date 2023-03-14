pub static PARAMETERISED_HEADERS: [&str; 2] = ["accept", "content-type"];
pub static SINGLE_VALUE_HEADERS: [&str; 7] = [
  "date",
  "accept-datetime",
  "if-modified-since",
  "if-unmodified-since",
  "expires",
  "retry-after",
  "last-modified"
];

/// Tries to parse the header value into multiple values, taking into account headers that should
/// not be split.
pub fn parse_header(name: &str, value: &str) -> Vec<String> {
  if SINGLE_VALUE_HEADERS.contains(&name.to_lowercase().as_str()) {
    vec![ value.trim().to_string() ]
  } else {
    value.split(',').map(|v| v.trim().to_string()).collect()
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;

  use crate::headers::parse_header;

  #[test]
  fn parse_simple_header_value() {
    let parsed = parse_header("X", "Y");
    expect!(parsed).to(be_equal_to(vec!["Y"]));
  }

  #[test]
  fn parse_multi_value_header_value() {
    let parsed = parse_header("Access-Control-Allow-Methods", "POST, GET, OPTIONS");
    expect!(parsed).to(be_equal_to(vec!["POST", "GET", "OPTIONS"]));
  }

  #[test]
  fn parse_multi_value_header_value_with_parameters() {
    let parsed = parse_header("accept", "text/html,application/xhtml+xml, application/xml;q=0.9,*/*; q=0.8");
    expect!(parsed).to(be_equal_to(vec!["text/html", "application/xhtml+xml", "application/xml;q=0.9", "*/*; q=0.8"]));
  }

  #[test]
  fn parse_known_single_value_header_value() {
    let parsed = parse_header("Last-Modified", "Mon, 01 Dec 2008 01:15:39 GMT");
    expect!(parsed).to(be_equal_to(vec!["Mon, 01 Dec 2008 01:15:39 GMT"]));
  }
}
