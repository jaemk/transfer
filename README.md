# Transfer [![Build Status](https://travis-ci.org/jaemk/transfer.svg?branch=master)](https://travis-ci.org/jaemk/transfer)

> Encrypted file transfer utility

Also see the command line client, [`transfer-cli`](https://github.com/jaemk/transfer-cli)


## Setup

- Server (base project dir):
    - `cargo install migrant --features postgresql`
    - `migrant init`
    - `migrant apply --all`
    - `cargo run -- serve`
- Webapp (`/web`):
    - `npm run dev`
    - open `http://localhost:8080`
    
