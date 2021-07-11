To run the php examples, the mock server DLL needs to be built using `cargo build` in the `rust/libpact_ffi` directory.

1. run `composer install`
2. run consumers
    1. `composer consumer-1-matches`
    2. `composer consumer-1-mismatches`
    3. `composer consumer-2-matches`
    4. `composer consumer-2-mismatches`
3. run provider
    1. `composer provider`

**NOTE:** This example needs to run on PHP >= 7.4.

To change the log level, use the `LOG_LEVEL` environment variable. I.e., to set
debug level: `LOG_LEVEL=debug composer consumer-1-matches`
