FROM rust:buster as planner

WORKDIR /app

RUN cargo install cargo-chef

COPY . /app

RUN cargo chef prepare --recipe-path recipe.json

# Stage 2 - build deps
FROM rust:buster as cacher

WORKDIR /app

RUN cargo install cargo-chef

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3 - build app
FROM rust:buster as builder

COPY . /app

WORKDIR /app

# Copy deps from cacher
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Build
RUN cargo build --release

# Stage 4 - run the app
FROM rust:buster as runner

WORKDIR /app

COPY --from=builder /app/target/release/server ./server

COPY ./lists ./lists

CMD ["./server"]