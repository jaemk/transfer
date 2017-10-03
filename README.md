# Transfer [![Build Status](https://travis-ci.org/jaemk/transfer.svg?branch=develop)](https://travis-ci.org/jaemk/transfer)

> Encrypted file transfer utility

Also see the command line client, [`transfer-cli`](https://github.com/jaemk/transfer-cli)


## Development

- Server:
    - Migration manager:
        - `cargo install migrant --features postgresql`
        - `migrant init`
        - `migrant apply --all`
    - `cargo run -- serve --port 3002`
    - configuration can be tweaked in `config.ron`
- Webapp (`/web`):
    - `yarn start`
    - open `http://localhost:3000`
    - api requests are proxied to `localhost:3002`


## Release Builds

> Cross compilation setup for the server `rust` executable

- Install [`docker`](https://www.digitalocean.com/community/tutorials/how-to-install-and-use-docker-on-ubuntu-16-04)
    - Add yourself to the `docker` group: `sudo usermod -a -G docker <user>`
    - Restart to pick up changes (logging in & out may suffice)
    - You should be able to run `docker version` without any errors
    - See `More on Docker and Cross` below for extra info
    - May need to start the Docker daemon if it's not already running: `sudo systemctl start docker` (not sure about windows/os-x)
- Install [`cross`](https://github.com/japaric/cross): `cargo install cross`
- Build server executables for targets listed in `build.py` script:
    - `build.py server`
- Build frontend app and copy bundled files to their static-file locations
    - `build.py web`

