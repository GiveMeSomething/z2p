FROM rust:1.78.0

WORKDIR /app

COPY . .

ENV SQLX_OFFLINE true
ENV APP_ENV production
RUN cargo build --release

ENTRYPOINT [ "./target/release/z2p" ]
