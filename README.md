# Transfer [![Build Status](https://travis-ci.org/jaemk/transfer.svg?branch=master)](https://travis-ci.org/jaemk/transfer)

> Encrypted file transfer utility

Also see the command line client, [`transfer-cli`](https://github.com/jaemk/transfer-cli)


## Dev Setup

- Server (base project dir):
    - `cargo install migrant --features postgresql`
    - `migrant init`
    - `migrant apply --all`
    - `cargo run -- serve --port 3002`
    - configuration can be tweaked in `config.ron`
- Webapp (`/web`):
    - `yarn start`
    - open `http://localhost:3000`

