###################################################
FROM rust:1.51.0-slim-buster as cargo-chef
WORKDIR /app
RUN cargo install cargo-chef

FROM cargo-chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM cargo-chef as cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json

FROM rust:1.51.0-slim-buster as dev
WORKDIR /app
RUN cargo install cargo-watch
COPY --from=cacher /app/target target
ENV RUST_LOG info
COPY . .
CMD ["cargo", "watch", "-x", "run"]

###################################################
FROM rust:1.51.0-slim-buster AS prod-builder
COPY . .
RUN cargo build --release

FROM debian:buster-slim AS prod
ENV RUST_LOG warn
COPY --from=prod-build /app/target/release/patab /usr/local/bin/patab
CMD ["patab"]
