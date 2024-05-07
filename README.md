![Logo of the project](https://raw.githubusercontent.com/pact-foundation/pact-core-mock-server/main/images/logo.svg)

[![Build](https://github.com/pact-foundation/pact-core-mock-server/actions/workflows/build.yml/badge.svg)](https://github.com/pact-foundation/pact-core-mock-server/actions/workflows/build.yml)

# Pact Core Mock Server

This project contains the in-process mock server for matching HTTP requests and generating responses from a pact file.
It implements the Pact specifications up to [V4 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-4).

There are two modules to this project:
* pact_mock_server - This is the library that provides the in-process mock server.
* pact_mock_server_cli - A CLI for starting and controlling seperate process mock servers.

## Usage

### Building

To build the libraries in this project, you need a working Rust environment.  Requires minimum Rust 1.77.0.
Refer to the [Rust Guide](https://www.rust-lang.org/learn/get-started).

The build tool used is `cargo`.

```shell
cargo build
```

This will compile all the libraries and put the generated files in `rust/target/debug`.

To run the tests:

```shell
cargo test
```

### Releasing

The released libraries for each module are built by a GH action that attaches the libraries to the GH release for each
crate. To release a crate, run the `release.groovy` script in the crate directory. This will guide you through the
release process for the crate. Then create a GH release using the tag and changelog created by the script.

## Contributing

See [CONTRIBUTING](CONTRIBUTING.md) (PRs are always welcome!).

## Documentation

Rust crate documentation is published to the Rust documentation site.

* [pact_mock_server](https://docs.rs/pact_mock_server/)
* [pact_mock_server_cli](https://docs.rs/pact_mock_server_cli/)

Additional documentation can be found at the main [Pact website](https://pact.io).

## Contact

Join us in slack: [![slack](https://slack.pact.io/badge.svg)](https://slack.pact.io)

or

- Twitter: [@pact_up](https://twitter.com/pact_up)
- Stack Overflow: [stackoverflow.com/questions/tagged/pact](https://stackoverflow.com/questions/tagged/pact)

## Licensing

The code in this project is licensed under a MIT license. See [LICENSE](LICENSE).
