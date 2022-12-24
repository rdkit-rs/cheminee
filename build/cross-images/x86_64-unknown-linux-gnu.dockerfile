FROM ghcr.io/assaydepot/x86_64-unknown-linux-gnu:0.3.0-assaydepot

ENV RDKIT_RELEASE=Release_2022_09_3 \
    CXX=g++ \
    DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y \
        curl cmake git \
        libeigen3-dev libssl-dev && \
    rm -rf /var/lib/apt/lists/*

ENV BOOST_VERSION_DOTS=1.81.0 \
    BOOST_VERSION_UNDERLINES=1_81_0

RUN cd /tmp; mkdir -p /opt/lib/aarch64-linux-gnu && \
    curl -LO https://boostorg.jfrog.io/artifactory/main/release/$BOOST_VERSION_DOTS/source/boost_$BOOST_VERSION_UNDERLINES.tar.gz && \
    tar xzf boost_$BOOST_VERSION_UNDERLINES.tar.gz
ADD x86_64-boost-user-config.jam /tmp/boost_$BOOST_VERSION_UNDERLINES/user-config.jam
RUN cd /tmp/boost_$BOOST_VERSION_UNDERLINES && \
    ./bootstrap.sh --prefix=/usr/aarch64-linux-gnu &&  \
    ./b2 target-os=linux toolset=gcc --user-config=./user-config.jam

RUN curl -vOL --silent https://github.com/rdkit/rdkit/archive/refs/tags/$RDKIT_RELEASE.tar.gz && \
    tar xzf $RDKIT_RELEASE.tar.gz && \
    cd rdkit-$RDKIT_RELEASE; mkdir -p build && cd build && \
     cmake .. -D CMAKE_CXX_COMPILER=$CXX \
              -D RDK_BUILD_PYTHON_WRAPPERS=OFF \
              -D RDK_OPTIMIZE_POPCNT=OFF \
              -D RDK_INSTALL_COMIC_FONTS=OFF \
              -D RDK_BUILD_FREETYPE_SUPPORT=OFF \
              -D RDK_INSTALL_STATIC_LIBS=ON \
              -D RDK_INSTALL_INTREE=OFF \
              -D RDK_BUILD_SWIG_JAVA_WRAPPER=OFF \
              -D RDK_BUILD_CPP_TESTS=OFF && \
     make install -j 4 && \
     rm -rf rdkit-$RDKIT_RELEASE $RDKIT_RELEASE.tar.gz