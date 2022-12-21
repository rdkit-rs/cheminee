FROM ghcr.io/assaydepot/aarch64-unknown-linux-gnu:0.3.0-assaydepot

RUN apt-get update && \
    apt-get install -y \
        binutils-aarch64-linux-gnu \
        curl cmake libboost-all-dev libeigen3-dev libssl-dev git && \
    rm -rf /var/lib/apt/lists/*

ENV LIBZ_SYS_STATIC=1 \
    PKG_CONFIG_ALLOW_CROSS=true \
    PKG_CONFIG_ALL_STATIC=true \
    X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC=1 \
    X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_DIR=/usr/local/musl/

ENV RDKIT_RELEASE=Release_2022_09_3 \
    CXX=g++ \
    DEBIAN_FRONTEND=noninteractive

RUN curl -OL --silent https://github.com/rdkit/rdkit/archive/refs/tags/$RELEASE.tar.gz; tar xzf $RELEASE.tar.gz
RUN cd rdkit-$RELEASE; mkdir -p build && cd build && \
     cmake .. -D CMAKE_CXX_COMPILER=$CXX \
              -D RDK_BUILD_PYTHON_WRAPPERS=OFF \
              -D RDK_OPTIMIZE_POPCNT=OFF \
              -D RDK_INSTALL_COMIC_FONTS=OFF \
              -D RDK_BUILD_FREETYPE_SUPPORT=OFF \
              -D RDK_INSTALL_STATIC_LIBS=ON \
              -D RDK_INSTALL_INTREE=OFF \
              -D RDK_BUILD_SWIG_JAVA_WRAPPER=OFF \
              -D RDK_BUILD_CPP_TESTS=OFF && \
     make install -j 4