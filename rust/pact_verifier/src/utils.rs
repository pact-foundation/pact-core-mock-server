//! Utility functions

use std::time::Duration;

use futures::StreamExt;
use reqwest::RequestBuilder;
use tokio::time::sleep;
use tracing::{trace, warn};

/// Retries a request on failure
pub(crate) async fn with_retries(retries: u8, request: RequestBuilder) -> Result<reqwest::Response, reqwest::Error> {
  match &request.try_clone() {
    None => {
      warn!("with_retries: Could not retry the request as it is not cloneable");
      request.send().await
    }
    Some(rb) => {
      futures::stream::iter((1..=retries).step_by(1))
        .fold((None::<Result<reqwest::Response, reqwest::Error>>, rb.try_clone()), |(response, request), attempt| {
          async move {
            match request {
              Some(request_builder) => match response {
                None => {
                  let next = request_builder.try_clone();
                  (Some(request_builder.send().await), next)
                },
                Some(response) => {
                  trace!("with_retries: attempt {}/{} is {:?}", attempt, retries, response);
                  match response {
                    Ok(ref res) => if res.status().is_server_error() {
                      match request_builder.try_clone() {
                        None => (Some(response), None),
                        Some(rb) => {
                          sleep(Duration::from_millis(10_u64.pow(attempt as u32))).await;
                          (Some(request_builder.send().await), Some(rb))
                        }
                      }
                    } else {
                      (Some(response), None)
                    },
                    Err(ref err) => if err.is_status() {
                      if err.status().unwrap_or_default().is_server_error() {
                        match request_builder.try_clone() {
                          None => (Some(response), None),
                          Some(rb) => {
                            sleep(Duration::from_millis(10_u64.pow(attempt as u32))).await;
                            (Some(request_builder.send().await), Some(rb))
                          }
                        }
                      } else {
                        (Some(response), None)
                      }
                    } else {
                      (Some(response), None)
                    }
                  }
                }
              }
              None => (response, None)
            }
          }
        }).await.0.unwrap()
    }
  }
}
