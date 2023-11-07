FROM rust:1.72 as build-stage

WORKDIR /vaalikoppi

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install musl-tools -y
RUN USER=root cargo new --bin vaalikoppi
RUN cargo install rsass-cli
COPY . .
RUN rsass src/static/scss/main.scss --style compressed > src/static/css/main.css
RUN cargo build --release --target x86_64-unknown-linux-musl


FROM scratch

WORKDIR /vaalikoppi

COPY --from=build-stage /vaalikoppi/target/x86_64-unknown-linux-musl/release/vaalikoppi /vaalikoppi/binary
CMD binary