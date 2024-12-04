FROM ubuntu:22.04

RUN apt-get update && apt-get install -y ca-certificates \
                                         libssl-dev \
                                         libboost-iostreams-dev \
                                         libboost-serialization-dev \
                                         curl \
                                         tar \
                                         unzip

RUN cd /tmp && \
    if [ $(dpkg --print-architecture) = "amd64" ]; then \
      curl -O https://rdkit-rs-debian.s3.eu-central-1.amazonaws.com/rdkit_2024_09_1_ubuntu_22_04_amd64.tar.gz; \
      tar xf rdkit_2024_9_1_ubuntu_22_04_amd64.tar.gz; \
    else \
      curl -O https://rdkit-rs-debian.s3.eu-central-1.amazonaws.com/rdkit_2024_09_1_ubuntu_22_04_arm64.tar.gz; \
      tar xf rdkit_2024_09_1_ubuntu_22_04_arm64.tar.gz; \
    fi; \
    mv /tmp/rdkit-Release_2024_09_1/Code /usr/local/include/rdkit; \
    mv /tmp/rdkit-Release_2024_09_1/build/lib/* /usr/lib/; \
    rm -rf /tmp/*

RUN cd /tmp && \
    if [ $(dpkg --print-architecture) = "amd64" ]; then \
      curl -O https://files.pythonhosted.org/packages/5b/00/af89cb211fc96ffdebb52a687dad7f83b0b1d82bc057f65309fa03a89911/tensorflow_cpu-2.15.1-cp311-cp311-manylinux_2_17_x86_64.manylinux2014_x86_64.whl; \
      unzip tensorflow_cpu-2.15.1-cp311-cp311-manylinux_2_17_x86_64.manylinux2014_x86_64.whl; \
    else \
      curl -O https://files.pythonhosted.org/packages/06/d5/05cd02db299821fd68ef5f8857506c21aeeddd024daf519d8643f0260952/tensorflow_cpu_aws-2.15.1-cp311-cp311-manylinux_2_17_aarch64.manylinux2014_aarch64.whl; \
      unzip tensorflow_cpu_aws-2.15.1-cp311-cp311-manylinux_2_17_aarch64.manylinux2014_aarch64.whl; \
      mv /tmp/tensorflow_cpu_aws.libs/libomp-54bf90fd.so.5 /usr/lib/; \
    fi; \
    mv /tmp/tensorflow/libtensorflow_cc.so.2 /usr/lib/; \
    mv /tmp/tensorflow/libtensorflow_framework.so.2 /usr/lib/; \
    ln -s /usr/lib/libtensorflow_cc.so.2 /usr/lib/libtensorflow_cc.so; \
    ln -s /usr/lib/libtensorflow_framework.so.2 /usr/lib/libtensorflow_framework.so; \
    ldconfig; \
    rm -rf /tmp/*

COPY target/release/cheminee /usr/local/bin/cheminee

CMD ["cheminee", "rest-api-server", "--bind=0.0.0.0:4001"]
