use std::collections::VecDeque;
use std::env;
use std::path::PathBuf;

use pact_models::pact::{ReadWritePact, write_pact};
use pact_models::prelude::Pact;
use pact_models::prelude::v4::V4Pact;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::sync_message::SynchronousMessage;
use pact_models::v4::V4InteractionType;
use tracing::{debug, error, info};

/// Iterator over the messages build with the PactBuilder
pub struct MessageIterator<MT> {
  pact: V4Pact,
  message_list: VecDeque<MT>,
  // Output directory to write pact files to when done
  output_dir: Option<PathBuf>,
}

/// Construct a new iterator over the asynchronous messages in the pact
pub fn asynchronous_messages_iter(pact: V4Pact) -> MessageIterator<AsynchronousMessage> {
  MessageIterator {
    pact: pact.clone(),
    message_list: pact.filter_interactions(V4InteractionType::Asynchronous_Messages)
      .iter()
      .map(|item| item.as_v4_async_message().unwrap())
      .collect(),
    output_dir: None
  }
}

/// Construct a new iterator over the synchronous messages in the pact
pub fn synchronous_messages_iter(pact: V4Pact) -> MessageIterator<SynchronousMessage> {
  MessageIterator {
    pact: pact.clone(),
    message_list: pact.filter_interactions(V4InteractionType::Synchronous_Messages)
      .iter()
      .map(|item| item.as_v4_sync_message().unwrap())
      .collect(),
    output_dir: None
  }
}

impl <MT> Iterator for MessageIterator<MT> {
  type Item = MT;

  fn next(&mut self) -> Option<Self::Item> {
    self.message_list.pop_front()
  }
}

impl <MT> Drop for MessageIterator<MT> {
  fn drop(&mut self) {
    if !::std::thread::panicking() {

      // Write out the Pact file
      let output_dir = self.output_dir.as_ref().map(|dir| dir.to_string_lossy().to_string())
        .unwrap_or_else(|| {
          let val = env::var("PACT_OUTPUT_DIR");
          debug!("env:PACT_OUTPUT_DIR = {:?}", val);
          val.unwrap_or_else(|_| "target/pacts".to_owned())
        });
      let overwrite = env::var("PACT_OVERWRITE");
      debug!("env:PACT_OVERWRITE = {:?}", overwrite);

      let pact_file_name = self.pact.default_file_name();
      let mut path = PathBuf::from(output_dir);
      path.push(pact_file_name);

      info!("Writing pact out to '{}'", path.display());
      let specification = self.pact.specification_version();
      if let Err(err) = write_pact(self.pact.boxed(), path.as_path(), specification,
                                   overwrite.unwrap_or_else(|_| String::default()) == "true") {
        error!("Failed to write pact to file - {}", err);
        panic!("Failed to write pact to file - {}", err);
      }
    }
  }
}
