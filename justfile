tailwind:
    npx tailwindcss -i styles/tailwind.css -o static/main.css

watch-tailwind:
    cargo watch -w templates/ -- just tailwind

prettier:
    npx prettier --write --ignore-unknown .

watch-cargo:
    cargo watch -w src/ -x run