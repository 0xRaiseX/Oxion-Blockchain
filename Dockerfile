FROM rust:latest

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN cargo build --release

CMD ["cargo", "run", "--release"]
