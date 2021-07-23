![Logo of the project](https://raw.githubusercontent.com/pact-foundation/pact-reference/master/images/logo.svg)

[![Pact-Rust Build](https://github.com/pact-foundation/pact-reference/workflows/Pact-Rust%20Build/badge.svg)](https://github.com/pact-foundation/pact-reference/actions?query=workflow%3A%22Pact-Rust+Build%22)

# Pact Reference Implementation
> Reference implementations for Pact Specification written in Rust

This project contains a reference implementation of the [Pact specifications](https://github.com/pact-foundation/pact-specification)
written in Rust, as well as example projects in JavaScript and C (and a few others) that use the mock server library.

## Usage

### Rust

For Rust projects, you can use the Rust crates from this library in your project directly. Refer to the [Rust project
readme](rust/README.md).

### Other languages

This project contains dynamic libraries that expose the core functionality through FFI (Foreign Function Interface).

For examples:
* [Javascript](javascript)
* [C](c/consumer-verification)
* [Ruby](ruby/example_consumer_spec)
* [PHP](php)

## Building

To build the libraries in this project, you need a working Rust environment. Refer to the [Rust Guide](https://www.rust-lang.org/learn/get-started).

The build tool used is `cargo`.

```shell
cd rust
cargo build
```

This will compile all the libraries and put the generated files in `rust/target/debug`.

## Contributing

See [CONTRIBUTING](CONTRIBUTING.md) (PRs are always welcome!).

## Documentation

Rust library documentation is published to the Rust documentation site. Refer to the [Rust project README](rust/README.md).

Additional documentation can be found at the main [Pact website](https://pact.io).

## Contact

Join us in slack: [![slack](https://slack.pact.io/badge.svg)](https://slack.pact.io)

or

- Twitter: [@pact_up](https://twitter.com/pact_up)
- Stack Overflow: [stackoverflow.com/questions/tagged/pact](https://stackoverflow.com/questions/tagged/pact)

## Licensing

The code in this project is licensed under a MIT license. See [LICENSE](LICENSE).
