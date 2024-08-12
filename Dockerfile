FROM ubuntu:22.04

RUN apt-get update && apt-get install -y ca-certificates libssl3 libboost-serialization1.74.0 curl

RUN cd /tmp && \
    if [ $(dpkg --print-architecture) = "amd64" ]; then \
      curl -O https://rdkit-rs-debian.s3.eu-central-1.amazonaws.com/rdkit_2024_03_3_ubuntu_14_04_amd64.tar.gz; \
      tar xf rdkit_2024_03_3_ubuntu_14_04_amd64.tar.gz; \
    else \
      curl -O https://rdkit-rs-debian.s3.eu-central-1.amazonaws.com/rdkit_2024_03_3_ubuntu_14_04_arm64.tar.gz; \
      tar xf rdkit_2024_03_3_ubuntu_14_04_arm64.tar.gz; \
    fi; \
    mv /tmp/rdkit-Release_2024_03_3/Code /usr/local/include/rdkit; \
    mv /tmp/rdkit-Release_2024_03_3/build/lib/* /usr/lib/

COPY target/release/cheminee /usr/local/bin/cheminee

CMD ["cheminee", "rest-api-server", "--bind=0.0.0.0:4001"]