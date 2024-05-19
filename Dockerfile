FROM rust:1.78.0

WORKDIR /app

COPY . .

RUN cargo build --release

ENTRYPOINT [ "./target/release/z2p" ]

