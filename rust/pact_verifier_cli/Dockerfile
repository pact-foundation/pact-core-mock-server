FROM ekidd/rust-musl-builder AS builder

# Add our source code.
ADD . ./

# Fix permissions on source code.
RUN sudo chown -R rust:rust /home/rust

RUN sudo apt-get update && sudo apt-get install -y libclang-dev

# Fix the cargo manifest so it can be built standalone
RUN sed -i -e 's/pact_verifier_ffi = {\s*version\s*=\s*"\([^"]*\).*/pact_verifier_ffi = "\1"/' Cargo.toml

# Build our application.
RUN cargo build --release

# Now, we need to build our _real_ Docker container, copying in the executable.
FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/pact_verifier_cli \
    /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/pact_verifier_cli"]
