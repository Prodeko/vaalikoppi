tailwind:
    bunx tailwindcss -i styles/tailwind.css -o static/main.css

watch-tailwind:
    cargo watch -w src/templates/ -- just tailwind

prettier:
    bunx prettier --write --ignore-unknown .

watch-cargo:
    cargo watch -w src/ -x run