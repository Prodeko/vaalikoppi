FROM rust:1.72
RUN rustup component add rustfmt
RUN USER=root cargo new --bin vaalikoppi
WORKDIR /vaalikoppi
RUN git config --global --add safe.directory /vaalikoppi
COPY ./Cargo.* .
RUN cargo build
COPY . .

