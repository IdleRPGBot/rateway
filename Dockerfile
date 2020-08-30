FROM docker.io/library/alpine:edge AS builder

RUN apk add --no-cache curl clang gcc musl-dev lld && \
    curl -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain nightly -y

ENV CC clang
ENV CFLAGS "-I/usr/lib/gcc/x86_64-alpine-linux-musl/10.2.0/"

WORKDIR /build
COPY . .

RUN set -ex && \
    rm /usr/bin/ld && \
    rm /usr/bin/cc && \
    ln -s /usr/bin/lld /usr/bin/ld && \
    ln -s /usr/bin/clang /usr/bin/cc && \
    source $HOME/.cargo/env && \
    cargo build --release && \
    strip /build/target/release/rateway

FROM docker.io/library/alpine:edge

RUN adduser -S rateway

USER rateway
WORKDIR /rateway

COPY --from=builder /build/target/release/rateway /rateway/run

CMD /rateway/run
