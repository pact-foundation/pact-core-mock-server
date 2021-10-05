To run the JavaScript examples, the pact_ffi Crate (which now also contains the
mock_server) first needs to be built using `cargo build` in the `rust/pact_ffi`
directory.

This should result in the appropriate library file(s) being created for your OS,
i.e. `rust/target/debug/libpact_ffi.[dll|so|dylib]`

1. run `npm install`
2. run `npm run simple_pact`

**NOTE:** This example needs to run on Node 10.

To change the log level, use the `RUST_LOG` environment variable. I.e., to set
debug level: `RUST_LOG=debug npm run simple_pact`

To run the failing example:

    $ npm run simple_pact_error
