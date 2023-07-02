FROM rust:1.69.0

WORKDIR /app

RUN apt update && apt install -y lld clang curl

COPY . .

ENV SQLX_OFFLINE true
RUN cargo build --release

ENTRYPOINT [ "./target/release/zero2prod" ]