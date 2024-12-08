# Vaalikoppi

## Electronic voting system for live elections

Vaalikoppi is designed for use in live meetings of small to medium organizations. The web application providers voters with a browser based mobile or desktop interface, and election officials with an administration UI meant for use with a desktop device.

After validating voting rights, a voter is given a temporary sign-in token with which they can authenticate as a voter.

Vaalikoppi currently supports the inclusive Gregory variant of single transferable vote.

## Hosting

Vaalikoppi currently only supports a single tenant. Therefore it has to be self hosted. Vaalikoppi requires one docker container and a Postgresql instance.

## Development

To run Vaalikoppi locally, first copy the file [.env.template](.env.template) to [.env](.env).

Vaalikoppi can be developed in a [dev container](https://containers.dev/). To run it in VSCode, install the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers), open the folder in VSCode and run the command `Dev Containers: Open Folder in Container`.

- Create an empty migration file: `sqlx migrate add <kuvaus_ilman_välilyöntejä> -r`
- Run the migrations: `sqlx migrate run`
- Revert one migration: `sqlx migrate revert`
- Build SCSS: `rsass /vaalikoppi/src/static/scss/main.scss --style compressed > /vaalikoppi/src/static/css/main.css`
- Install dependencies needed by Playwright: `npx playwright install --with-deps`
- Run Playwright: `npx playwright test`
- Run Playwright in headed mode: `npx playwright test --headed`

## Tech

Vaalikoppi was originally developed with Django. The backend was completely rewritten in Rust in 2023.

Vaalikoppi uses the following technologies:
- Rust
- Tokio
- Axum
- Askama
- Postgresql

## Roadmap

The earliest version of Vaalikoppi was developed in 2017. The Rust-based version was published in 2023. The next major major step is a rewrite of the frontend.

Future development directions include _multi-tenant_ support, integration into different authentication services for admin sign-in and cryptographic anonymity with, for example, _linkable ring signatures_.