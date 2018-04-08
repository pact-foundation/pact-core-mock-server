#!/bin/bash

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-docker.sh version"
  exit 1
fi

docker build . -t pactfoundation/pact-ref-verifier:$1
docker push pactfoundation/pact-ref-verifier:$1
docker tag pactfoundation/pact-ref-verifier:$1 pactfoundation/pact-ref-verifier:latest
docker push pactfoundation/pact-ref-verifier:latest
