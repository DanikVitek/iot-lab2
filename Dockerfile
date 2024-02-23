FROM lukemathwalker/cargo-chef:latest-rust-1.76.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM debian:bookworm-slim as runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates openssl \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/lab2 lab2
COPY migrations migrations
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./lab2" ]