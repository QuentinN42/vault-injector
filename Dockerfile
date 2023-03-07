FROM rust as build

RUN apt update
RUN apt install -y musl-tools

# ------------------------------- Build OpenSSL for the `musl` build target
RUN \
  ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm && \
  ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic && \
  ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux

WORKDIR /musl

RUN wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz
RUN tar zxvf OpenSSL_1_1_1f.tar.gz 
WORKDIR /musl/openssl-OpenSSL_1_1_1f/

RUN CC="musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/musl --openssldir=/musl/ssl linux-x86_64
RUN make depend
RUN make -j$(nproc)
RUN make install
# -------------------------------

WORKDIR /build

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml .
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs

ENV OPENSSL_DIR=/musl

RUN cargo build --target=x86_64-unknown-linux-musl
RUN cargo build --target=x86_64-unknown-linux-musl --release
RUN rm src/*.rs

COPY src ./src
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target=x86_64-unknown-linux-musl --release

FROM scratch
USER 1000
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/vault-injector /prog
CMD ["/prog"]
ENV RUST_LOG=vault_injector
