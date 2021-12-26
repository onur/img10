FROM rust:1.57
WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl

COPY . /app
RUN cargo build --release --target x86_64-unknown-linux-musl && \
    strip /app/target/x86_64-unknown-linux-musl/release/supershare

FROM gcr.io/distroless/static-debian11
COPY --from=0 /app/target/x86_64-unknown-linux-musl/release/supershare /

ENV RUST_LOG=info
CMD ["./supershare"]
