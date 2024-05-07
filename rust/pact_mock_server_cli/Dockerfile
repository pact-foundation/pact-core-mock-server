ARG ARCH=
FROM rust:1-alpine3.15 AS builder
ARG BIN_ARCH=x86_64
ARG VERSION=1.0.5

RUN apk --no-cache add wget
RUN wget -O pact_mock_server_cli.gz https://github.com/pact-foundation/pact-reference/releases/download/pact_mock_server_cli-v${VERSION}/pact_mock_server_cli-linux-${BIN_ARCH}.gz
RUN gunzip pact_mock_server_cli.gz
RUN chmod +x pact_mock_server_cli


# Now, we need to build our _real_ Docker container, copying in the executable.
FROM ${ARCH}alpine:3.16
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /pact_mock_server_cli \
    /usr/local/bin/

EXPOSE 8080 9000-9020

ENTRYPOINT ["/usr/local/bin/pact_mock_server_cli"]
CMD ["start", "--base-port", "9000"]
