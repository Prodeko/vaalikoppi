FROM rust:1.72 as build-stage


WORKDIR /vaalikoppi

ARG DATABASE_URL

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install musl-tools -y
RUN USER=root cargo new --bin vaalikoppi
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres && cargo install rsass-cli
COPY . .
RUN sqlx migrate run
RUN rsass src/static/scss/main.scss --style compressed > src/static/css/main.css
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:1.72,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

WORKDIR /vaalikoppi

COPY --from=build-stage /vaalikoppi/target/x86_64-unknown-linux-musl/release/vaalikoppi /vaalikoppi/target/x86_64-unknown-linux-musl/release/vaalikoppi
COPY --from=build-stage /vaalikoppi/src/static /vaalikoppi/src/static
CMD /vaalikoppi/target/x86_64-unknown-linux-musl/release/vaalikoppi