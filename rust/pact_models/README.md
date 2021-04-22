# Pact Models

This library provides the core models for dealing with Pact files. It supports the
[V3 pact specification](https://github.com/pact-foundation/pact-specification/tree/version-3) and
[V4 pact specification](https://github.com/pact-foundation/pact-specification/tree/version-4).

[Online rust docs](https://docs.rs/pact_models/)

## Reading and writing Pact files

The `Pact` struct has methods to read and write pact JSON files. It supports all the specification
versions up to V4, but will convert a V1, V1.1 and V2 spec file to a V3 format.
