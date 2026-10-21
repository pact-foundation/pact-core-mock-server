# Packages the static musl pact-mock-server from a dist release. No step runs
# on the target platform, so a multi-platform build needs no emulation.
FROM --platform=$BUILDPLATFORM alpine:3.22 AS fetch
ARG VERSION
ARG TARGETARCH
RUN apk add --no-cache ca-certificates curl xz
WORKDIR /dist
RUN case "$TARGETARCH" in \
        amd64) target=x86_64-unknown-linux-musl ;; \
        arm64) target=aarch64-unknown-linux-musl ;; \
        *) echo "unsupported architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && curl -fsSL "https://github.com/pact-foundation/pact-core-mock-server/releases/download/pact_mock_server_cli-v${VERSION}/pact_mock_server_cli-${target}.tar.xz" \
       | tar xJ --strip-components=1 \
    && install -m 0755 pact-mock-server /pact-mock-server

FROM alpine:3.22
COPY --from=fetch /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=fetch /pact-mock-server /usr/local/bin/pact-mock-server

EXPOSE 8080 9000-9020

ENTRYPOINT ["/usr/local/bin/pact-mock-server"]
CMD ["start", "--base-port", "9000"]
