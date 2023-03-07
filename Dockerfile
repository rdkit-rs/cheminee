FROM ubuntu:22.04 AS builder

RUN apt-get update; apt-get install -y ca-certificates curl; \
    echo 'deb [trusted=yes] https://rdkit-rs-debian.s3.amazonaws.com jammy main' > /etc/apt/sources.list.d/rdkit-rs.list && \
    apt-get update && apt-get install -y build-essential librdkit-dev libssl-dev libboost1.74-dev libboost-serialization1.74-dev pkg-config

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=1.67 -y

ADD . /code

RUN cd /code; . ~/.cargo/env; cargo build --release

FROM ubuntu:22.04

COPY --from=builder /code/target/release/cheminee /usr/local/bin/cheminee

CMD ["cheminee", "rest-api-server"]
