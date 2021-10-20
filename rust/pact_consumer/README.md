# Pact test DSL for writing consumer pact tests in Rust

This library provides a test DSL for writing consumer pact tests in Rust. It supports the
[V3 pact specification](https://github.com/pact-foundation/pact-specification/tree/version-3) and
[V4 pact specification](https://github.com/pact-foundation/pact-specification/tree/version-4).

[Online rust docs](https://docs.rs/pact_consumer/)

## To use it

To use it, add it to your dev-dependencies in your cargo manifest:

```
[dev-dependencies]
pact_consumer = "0.8.0-beta.4"
```

You can now write a pact test using the consumer DSL.

```rust
use pact_consumer::prelude::*;
use pact_consumer::*;

#[test]
fn a_service_consumer_side_of_a_pact_goes_a_little_something_like_this() {

    // Define the Pact for the test (you can setup multiple interactions by chaining the given or upon_receiving calls)
    let pact_runner = ConsumerPactBuilder::consumer("Consumer".to_string()) // Define the service consumer by name
        .has_pact_with("Alice Service".to_string())                         // Define the service provider that it has a pact with
        .given("there is some good mallory".to_string())                    // defines a provider state. It is optional.
        .upon_receiving("a retrieve Mallory request".to_string())           // upon_receiving starts a new interaction
            .path(s!("/mallory"))                                           // define the request, a GET (default) request to '/mallory'
        .will_respond_with()                                                // define the response we want returned
            .status(200)
            .headers(hashmap!{ "Content-Type".to_string() => "text/html".to_string() })
            .body(OptionalBody::Present("That is some good Mallory.".to_string()))
        .build();

    // Execute the run method to have the mock server run (the URL to the mock server will be passed in).
    // It takes a closure to execute your requests and returns a Pact VerificationResult.
    let result = pact_runner.run(&|url| {
        let client = Client { url: url.clone(), .. Client::default() }; // You would use your actual client code here
        let result = client.fetch("/mallory"); // we get our client code to execute the request
        expect!(result).to(be_ok().value("That is some good Mallory."));
        Ok(())
    });
    expect!(result).to(be_equal_to(VerificationResult::PactVerified)); // This means it is all good
}
```

### Changing the output directory

By default, the pact files will be written to `target/pacts`. To change this, set the environment variable `PACT_OUTPUT_DIR`.

### Forcing pact files to be overwritten

Pacts are merged with existing pact files when written. To change this behaviour so that the files
are always overwritten, set the environment variable `PACT_OVERWRITE` to `true`.

## Testing messages

Testing message consumers is supported. There are two types: asynchronous messages and synchronous request/response.

### Asynchronous messages

Asynchronous messages are you normal type of single shot or fire and forget type messages. They are typically sent to a
message queue or topic as a notification or event. With Pact tests, we will be testing that our consumer of the messages
works with the messages setup as the expectations in test. This should be the message handler code that processes the
actual messages that come off the message queue in production.

The generated Pact file from the test run can then be used to verify whatever created the messages adheres to the Pact
file.

```rust
use pact_consumer::prelude::*;
use pact_consumer::*;

#[tokio::test]
async fn a_message_consumer_side_of_a_pact_goes_a_little_something_like_this() {

    // Define the Pact for the test (you can setup multiple interactions by chaining the given or message_interaction calls)
    // For messages we need to use the V4 Pact format.
    let pact_builder = PactBuilder::PactBuilder::new_v4("message-consumer", "message-provider"); // Define the message consumer and provider by name
    pact_builder
      // defines a provider state. It is optional.
      .given("there is some good mallory".to_string())                                           
      // Adds an interaction given the message description and type.
      .message_interaction("Mallory Message", "core/interaction/message", |mut i| async move { 
        // Can set the test name (optional)
        i.test_name("a_message_consumer_side_of_a_pact_goes_a_little_something_like_this");
        // Set the contents of the message. Here we use a JSON pattern, so that matching rules are applied
        i.json_body(json_pattern!({
          "mallory": like!("That is some good Mallory.")
        }));
        // Need to return the mutated interaction builder
        i
      })
      .await;

    // This will return each message configured with the Pact builder. We need to process them
    // with out message handler (it should be the one used to actually process your messages).
    for message in pact_builder.messages() {
      let bytes = message.contents.contents.value().unwrap();
      
      // Process the message here as it would if it came off the queue
      let message: Value = serde_json::from_slice(&bytes);      

      // Make some assertions on the processed value
      expect!(message.as_object().unwrap().get("mallory")).to(be_some().value());
    }
}
```

### Synchronous request/response messages

## Using Pact plugins

