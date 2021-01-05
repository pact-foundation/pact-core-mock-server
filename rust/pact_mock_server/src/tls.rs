//! TLS support structs

// Copyright (c) 2018 Sean McArthur (https://github.com/seanmonstar/warp/blob/master/src/tls.rs)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::fs::File;
use std::io::{self, BufReader, Cursor, Read};
use std::path::{Path, PathBuf};

use tokio_rustls::rustls::{NoClientAuth, ServerConfig, TLSError};

/// Represents errors that can occur building the TlsConfig
#[derive(Debug)]
pub enum TlsConfigError {
  /// IO Error
  Io(io::Error),
  /// An Error parsing the Certificate
  CertParseError,
  /// An Error parsing a Pkcs8 key
  Pkcs8ParseError,
  /// An Error parsing a Rsa key
  RsaParseError,
  /// An error from an empty key
  EmptyKey,
  /// An error from an invalid key
  InvalidKey(TLSError),
}

impl std::fmt::Display for TlsConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TlsConfigError::Io(err) => err.fmt(f),
      TlsConfigError::CertParseError => write!(f, "certificate parse error"),
      TlsConfigError::Pkcs8ParseError => write!(f, "pkcs8 parse error"),
      TlsConfigError::RsaParseError => write!(f, "rsa parse error"),
      TlsConfigError::EmptyKey => write!(f, "key contains no private key"),
      TlsConfigError::InvalidKey(err) => write!(f, "key contains an invalid key, {}", err),
    }
  }
}

impl std::error::Error for TlsConfigError {}

/// Builder to set the configuration for the Tls server.
pub struct TlsConfigBuilder {
  cert: Box<dyn Read + Send + Sync>,
  key: Box<dyn Read + Send + Sync>,
}

impl std::fmt::Debug for TlsConfigBuilder {
  fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    f.debug_struct("TlsConfigBuilder").finish()
  }
}

impl TlsConfigBuilder {
  /// Create a new TlsConfigBuilder
  pub fn new() -> TlsConfigBuilder {
    TlsConfigBuilder {
      key: Box::new(io::empty()),
      cert: Box::new(io::empty()),
    }
  }

  /// sets the Tls key via File Path, returns `TlsConfigError::IoError` if the file cannot be open
  pub fn key_path(mut self, path: impl AsRef<Path>) -> Self {
    self.key = Box::new(LazyFile {
      path: path.as_ref().into(),
      file: None,
    });
    self
  }

  /// sets the Tls key via bytes slice
  pub fn key(mut self, key: &[u8]) -> Self {
    self.key = Box::new(Cursor::new(Vec::from(key)));
    self
  }

  /// Specify the file path for the TLS certificate to use.
  pub fn cert_path(mut self, path: impl AsRef<Path>) -> Self {
    self.cert = Box::new(LazyFile {
      path: path.as_ref().into(),
      file: None,
    });
    self
  }

  /// sets the Tls certificate via bytes slice
  pub fn cert(mut self, cert: &[u8]) -> Self {
    self.cert = Box::new(Cursor::new(Vec::from(cert)));
    self
  }

  /// Build the TLS configuration
  pub fn build(mut self) -> Result<ServerConfig, TlsConfigError> {
    let mut cert_rdr = BufReader::new(self.cert);
    let cert = tokio_rustls::rustls::internal::pemfile::certs(&mut cert_rdr)
      .map_err(|()| TlsConfigError::CertParseError)?;

    let key = {
      // convert it to Vec<u8> to allow reading it again if key is RSA
      let mut key_vec = Vec::new();
      self.key
        .read_to_end(&mut key_vec)
        .map_err(TlsConfigError::Io)?;

      if key_vec.is_empty() {
        return Err(TlsConfigError::EmptyKey);
      }

      let mut pkcs8 = tokio_rustls::rustls::internal::pemfile::pkcs8_private_keys(
        &mut key_vec.as_slice(),
      )
        .map_err(|()| TlsConfigError::Pkcs8ParseError)?;

      if !pkcs8.is_empty() {
        pkcs8.remove(0)
      } else {
        let mut rsa = tokio_rustls::rustls::internal::pemfile::rsa_private_keys(
          &mut key_vec.as_slice(),
        )
          .map_err(|()| TlsConfigError::RsaParseError)?;

        if !rsa.is_empty() {
          rsa.remove(0)
        } else {
          return Err(TlsConfigError::EmptyKey);
        }
      }
    };

    let mut config = ServerConfig::new(NoClientAuth::new());
    config
      .set_single_cert(cert, key)
      .map_err(|err| TlsConfigError::InvalidKey(err))?;
    config.set_protocols(&["h2".into(), "http/1.1".into()]);
    Ok(config)
  }
}

struct LazyFile {
  path: PathBuf,
  file: Option<File>,
}

impl LazyFile {
  fn lazy_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    if self.file.is_none() {
      self.file = Some(File::open(&self.path)?);
    }

    self.file.as_mut().unwrap().read(buf)
  }
}

impl Read for LazyFile {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.lazy_read(buf).map_err(|err| {
      let kind = err.kind();
      io::Error::new(
        kind,
        format!("error reading file ({:?}): {}", self.path.display(), err),
      )
    })
  }
}
