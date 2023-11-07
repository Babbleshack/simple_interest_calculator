FROM rust:1.73 as builder

WORKDIR /usr/src/loan_calculator

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /usr/src/loan_calculator/target/release/oneiro .

ENTRYPOINT ["./oneiro"]
