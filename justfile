tailwind:
    bunx tailwindcss -i src/styles/tailwind.css -o src/static/tailwind.css

watch-tailwind:
    cargo watch -w src/templates/ -- just tailwind

prettier:
    bunx prettier --write --ignore-unknown .

watch-cargo:
    cargo watch -w src/ -x run