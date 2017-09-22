# Transfer

> Encrypted file transfer utility

> Built with `rust` and `vue.js`.

Also see the command line client, [`transfer-cli`](https://github.com/jaemk/transfer-cli)

- Server (base project dir):
    - `cargo install migrant --features postgresql`
    - `migrant init` / `migrant apply --all`
    - Nightly `rust` is currently required for `rocket`
    - `rustup override set nightly`
    - `cargo run -- serve --log`
- Webapp (`/web`):
    - `npm run dev`
    - open `http://localhost:8080`
    

