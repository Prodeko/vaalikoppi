FROM rust:1.81

# Install bun.js
RUN curl -fsSL https://bun.sh/install | bash

RUN rustup target add x86_64-unknown-linux-musl

# Add nodesource PPA
RUN curl -fsSL https://deb.nodesource.com/setup_23.x | bash -s
RUN apt-get update && apt-get install musl-tools nodejs -y    

RUN rustup component add rustfmt
RUN USER=root cargo new --bin vaalikoppi
WORKDIR /vaalikoppi
RUN git config --global --add safe.directory /vaalikoppi

RUN cargo install cargo-watch
RUN cargo install just

# Install SQLx CLI for database migrations (see README)
RUN cargo install sqlx-cli@0.8.3 --no-default-features --features native-tls,postgres && cargo install rsass-cli@0.29.0

# Install dependencies for Playwright tests
COPY package.json package-lock.json ./
RUN npm install && npx playwright install --with-deps

ENTRYPOINT tail -f /dev/null