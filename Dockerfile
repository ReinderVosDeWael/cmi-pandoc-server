FROM rust:1.81-slim as builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM pandoc/core

WORKDIR /app
EXPOSE 3000

COPY --from=builder /build/target/release/handler .

CMD ["handler"]
