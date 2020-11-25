FROM docker.io/library/alpine:edge AS builder

RUN apk add --no-cache curl clang gcc musl-dev lld cmake make && \
    curl -sSf https://sh.rustup.rs | sh -s -- --profile minimal --default-toolchain nightly -y

ENV CC clang
ENV CFLAGS "-I/usr/lib/gcc/x86_64-alpine-linux-musl/10.2.1/ -L/usr/lib/gcc/x86_64-alpine-linux-musl/10.2.1/"

RUN rm /usr/bin/ld && \
    rm /usr/bin/cc && \
    ln -s /usr/bin/lld /usr/bin/ld && \
    ln -s /usr/bin/clang /usr/bin/cc && \
    ln -s /usr/lib/gcc/x86_64-alpine-linux-musl/10.2.1/crtbeginS.o /usr/lib/crtbeginS.o && \
    ln -s /usr/lib/gcc/x86_64-alpine-linux-musl/10.2.1/crtendS.o /usr/lib/crtendS.o

WORKDIR /build

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./.cargo ./.cargo

RUN mkdir src/
RUN echo 'fn main() {}' > ./src/main.rs
RUN source $HOME/.cargo/env && \
    cargo build --release

RUN rm -f target/release/deps/rateway*

COPY ./src ./src

RUN source $HOME/.cargo/env && \
    cargo build --release && \
    strip /build/target/release/rateway

FROM docker.io/library/alpine:edge

RUN adduser -S rateway

USER rateway
WORKDIR /rateway

COPY --from=builder /build/target/release/rateway /rateway/run

CMD /rateway/run
