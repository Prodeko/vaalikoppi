FROM rust:1.72
RUN USER=root cargo new --bin vaalikoppi
WORKDIR /vaalikoppi
RUN git config --global --add safe.directory /vaalikoppi
COPY ./Cargo.* .
RUN cargo build
COPY . .

