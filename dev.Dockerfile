FROM rust:1.81

# Workaround for issue with postgres vscode extension https://github.com/microsoft/vscode-postgresql/issues/77
RUN wget http://mirrors.kernel.org/ubuntu/pool/main/libf/libffi/libffi6_3.2.1-8_amd64.deb && \
    apt install ./libffi6_3.2.1-8_amd64.deb

# installs fnm (Fast Node Manager)
ARG version=22

RUN curl -fsSL https://fnm.vercel.app/install | bash -s -- --install-dir './fnm' \
    && cp ./fnm/fnm /usr/bin && fnm install $version

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install musl-tools -y    

RUN rustup component add rustfmt
RUN USER=root cargo new --bin vaalikoppi
WORKDIR /vaalikoppi
RUN git config --global --add safe.directory /vaalikoppi

# Install SQLx CLI for database migrations (see README)
RUN cargo install sqlx-cli@0.7.3 --locked --no-default-features --features native-tls,postgres && cargo install rsass-cli@0.28.8 --locked
# No need to copy or build anything in dev container
# COPY ./Cargo.* .
# RUN cargo build
# COPY . .

ENTRYPOINT tail -f /dev/null