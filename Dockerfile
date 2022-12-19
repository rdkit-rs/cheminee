FROM cheminee:base AS builder

ADD . /code
RUN cd /code; . ~/.cargo/env; cargo build --release

FROM ubuntu:22.04

COPY --from=builder /code/target/release/cheminee /usr/local/bin/cheminee

CMD ["cheminee", "rest-api-server"]
