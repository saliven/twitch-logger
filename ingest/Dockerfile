FROM rust:buster AS planner

WORKDIR /app

RUN cargo install cargo-chef

COPY . /app

RUN cargo chef prepare --bin ingest --recipe-path recipe.json

# Stage 2 - build deps
FROM rust:buster AS cacher

WORKDIR /app

RUN cargo install cargo-chef

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --bin ingest --release --recipe-path recipe.json

# Stage 3 - build app
FROM rust:buster AS builder

COPY . /app

WORKDIR /app

# Copy deps from cacher
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Build
RUN cargo build --bin ingest --release

# Stage 4 - run the app
FROM rust:buster AS runner

WORKDIR /app

COPY --from=builder /app/target/release/ingest ./ingest

CMD ["./ingest"]
