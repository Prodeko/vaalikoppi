FROM rust:1.72

# Workaround for issue with postgres vscode extension https://github.com/microsoft/vscode-postgresql/issues/77
RUN wget http://mirrors.kernel.org/ubuntu/pool/main/libf/libffi/libffi6_3.2.1-8_amd64.deb && \
    apt install ./libffi6_3.2.1-8_amd64.deb
    

RUN rustup component add rustfmt
RUN USER=root cargo new --bin vaalikoppi
WORKDIR /vaalikoppi
RUN git config --global --add safe.directory /vaalikoppi

# Install SQLx CLI for database migrations (see README)
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres && cargo install rsass-cli
# No need to copy or build anything in dev container
# COPY ./Cargo.* .
# RUN cargo build
# COPY . .

