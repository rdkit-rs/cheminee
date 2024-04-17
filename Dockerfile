FROM ubuntu:22.04

RUN apt-get update && apt-get install -y ca-certificates wget && \
          wget --quiet https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O ~/miniconda3/miniconda.sh && \
          bash ~/miniconda3/miniconda.sh -b -u -p ~/miniconda3 && \
          rm -rf ~/miniconda3/miniconda.sh && \
          ~/miniconda3/bin/conda install conda-forge::rdkit && \
          sudo apt-get update && \
          sudo apt-get install -y build-essential libssl-dev libboost1.74-dev libboost-serialization1.74-dev pkg-config && \
    apt-get update && apt-get install -y libssl3 libboost-serialization1.74.0

COPY target/release/cheminee /usr/local/bin/cheminee

CMD ["cheminee", "rest-api-server"]