#[macro_use] extern crate helix;
extern crate pact_mock_server;
extern crate pact_matching;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate uuid;

use simplelog::*;
use uuid::Uuid;
use pact_matching::models::Pact;

ruby! {
  class PactMockServerMk2 {
      def create_mock_server(pact_json: String, port: i32) -> Result<i32, String> {
        SimpleLogger::init(LogLevelFilter::Info, Config::default()).unwrap();

        match serde_json::from_str(&pact_json) {
          Ok(pact_json) => {
            let pact = Pact::from_json(&"<create_mock_server>".to_string(), &pact_json);
            pact_mock_server::start_mock_server(Uuid::new_v4().simple().to_string(), pact, port)
              .map_err(|err| {
                error!("Could not start mock server: {}", err);
                format!("Could not start mock server: {}", err)
              })
          },
          Err(err) => {
            error!("Could not parse pact json: {}", err);
            Err(format!("Could not parse pact json: {}", err))
          }
        }
      }

      def cleanup_mock_server(port: i32) -> bool {
        pact_mock_server::shutdown_mock_server_by_port(port)
      }

      def all_matched(port: i32) -> bool {
        pact_mock_server::mock_server_matched(port)
      }
  }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
