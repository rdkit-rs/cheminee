FROM ghcr.io/assaydepot/aarch64-unknown-linux-gnu:0.3.0-assaydepot

ENV LIBZ_SYS_STATIC=1 \
    PKG_CONFIG_ALLOW_CROSS=true \
    PKG_CONFIG_ALL_STATIC=true \
    X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=1 \
    X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_DIR=/usr/local/musl/

ENV RDKIT_RELEASE=Release_2022_09_3 \
    CXX=aarch64-linux-gnu-g++ \
    CC=aarch64-linux-gnu-gcc \
    AR=aarch64-linux-gnu-ar \
    DEBIAN_FRONTEND=noninteractive

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y \
        binutils-aarch64-linux-gnu \
        curl cmake git \
        aptitude && \
    aptitude install -y \
        libeigen3-dev:arm64 libssl-dev:arm64 && \
    rm -rf /var/lib/apt/lists/*

RUN cd /tmp; mkdir -p /opt/lib/aarch64-linux-gnu && \
    curl -LO https://boostorg.jfrog.io/artifactory/main/release/1.78.0/source/boost_1_78_0.tar.gz && \
    tar xzf boost_1_78_0.tar.gz
ADD aarch64-boost-user-config.jam /tmp/boost_1_78_0/user-config.jam
RUN cd /tmp/boost_1_78_0 && \
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