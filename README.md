# Prodeko äänestyskilke :bookmark:

## Komennot

- Tyhjän migraation saa luotua komennolla `sqlx migrate add <kuvaus_ilman_välilyöntejä> -r`
- Aja migraatiot: `sqlx migrate run`
- Peruuta migraatio: `sqlx migrate revert`
- Luo SQL-kyselyistä tyypit CI:tä varten `cargo sqlx prepare -- --release --all-targets --all-features`
- Buildaa SCSS: `rsass /vaalikoppi/src/static/scss/main.scss --style compressed > /vaalikoppi/src/static/css/main.css`
