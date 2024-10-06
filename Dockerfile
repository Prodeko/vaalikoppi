FROM rust:1.81 as build-stage


WORKDIR /vaalikoppi

ARG DATABASE_URL

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install musl-tools -y
RUN USER=root cargo new --bin vaalikoppi
RUN cargo install sqlx-cli@0.7.3 --locked --no-default-features --features native-tls,postgres && cargo install rsass-cli@0.28.8 --locked
COPY . .
RUN sqlx migrate run
RUN rsass src/static/scss/main.scss --style compressed > src/static/css/main.css
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:1.81,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release --target x86_64-unknown-linux-musl && cp target/x86_64-unknown-linux-musl/release/vaalikoppi output_binary

FROM scratch

WORKDIR /vaalikoppi

COPY --from=build-stage /vaalikoppi/output_binary /vaalikoppi/output_binary
COPY --from=build-stage /vaalikoppi/src/static /vaalikoppi/src/static
CMD /vaalikoppi/output_binary