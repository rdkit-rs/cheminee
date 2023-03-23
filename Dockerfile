FROM ubuntu:22.04

RUN apt-get update && apt-get install -y ca-certificates && \
    echo 'deb [trusted=yes] https://rdkit-rs-debian.s3.amazonaws.com jammy main' > /etc/apt/sources.list.d/rdkit-rs.list && \
    apt-get update && apt-get install -y librdkit1 libssl3 libboost-serialization1.74.0

COPY target/release/cheminee /usr/local/bin/cheminee

CMD ["cheminee", "rest-api-server"]