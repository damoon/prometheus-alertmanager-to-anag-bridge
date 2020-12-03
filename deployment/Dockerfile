FROM rust:1.48.0-slim-buster AS rust
WORKDIR /app

###################################################
FROM rust as planner
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust as cacher
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as dev
RUN cargo install cargo-watch
COPY . .
COPY --from=cacher /app/target target
CMD ["cargo", "watch", "-x", "run"]

###################################################
FROM rust AS prod-builder
COPY . .
RUN cargo build --release

FROM debian:buster-slim AS prod
COPY --from=prod-build /app/target/release/patab /usr/local/bin/patab
CMD ["patab"]
