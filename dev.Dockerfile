FROM rust:1.83

RUN rustup target add x86_64-unknown-linux-musl

# Add nodesource PPA
RUN curl -fsSL https://deb.nodesource.com/setup_23.x | bash -s
RUN apt-get update && apt-get install musl-tools nodejs git -y    

RUN rustup component add rustfmt
RUN USER=root cargo new --bin vaalikoppi
WORKDIR /vaalikoppi
RUN git config --global --add safe.directory /vaalikoppi

# Install SQLx CLI for database migrations (see README)
RUN cargo install sqlx-cli@0.7.3 --locked --no-default-features --features native-tls,postgres && cargo install rsass-cli@0.28.8 --locked

# Install dependencies for Playwright tests
COPY package.json package-lock.json ./
RUN npm install && npx playwright install --with-deps

ENTRYPOINT tail -f /dev/null