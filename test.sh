#!/bin/bash

set -ex

cargo fmt
cargo clippy -- -D warnings
cargo build
cargo build --release

USER="theempty"
NAME="lci-gateway-exporter"
TEST_REPO="192.168.7.7:5000"

sed -E -i .bak 's/ENV RUST_LOG=.+$/ENV RUST_LOG=trace/' Dockerfile
docker build -t ${TEST_REPO}/${USER}/${NAME} .
docker push ${TEST_REPO}/${USER}/${NAME}
kubectl rollout restart deployment/${NAME}
sleep 45
kubectl logs -f -l app=${NAME}