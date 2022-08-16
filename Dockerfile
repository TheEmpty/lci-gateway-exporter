# Not working on Alpine atm
# FROM alpine:3.16
FROM rust:1.63-slim-buster

# Alpine block
# RUN apk add --no-cache rustup gcc openssl-dev musl-dev \
#   && rustup-init -y --default-toolchain 1.63.0 \
#   && mkdir -p /tmp/lci-gateway-exporter

RUN apt-get update
RUN apt-get install -y gcc libssl-dev pkg-config
RUN mkdir -p /tmp/lci-gateway-exporter
COPY src /tmp/lci-gateway-exporter/src
COPY Cargo.toml /tmp/lci-gateway-exporter

RUN cd /tmp/lci-gateway-exporter \
  && cargo build --release --verbose \
  && cp target/release/lci-gateway-exporter /opt \
  && rm -fr /src
# more alpine stuff
#   && apk --purge del rustup gcc \
#   && rm -fr ~/.cargo ~/.rustup

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

ENTRYPOINT ["/opt/lci-gateway-exporter"]
