# Transfer [![Build Status](https://travis-ci.org/jaemk/transfer.svg?branch=develop)](https://travis-ci.org/jaemk/transfer)

> Encrypted file transfer utility

Also see the command line client, [`transfer-cli`](https://github.com/jaemk/transfer-cli)


## Development

- Backend:
    - Install [`rust`](https://www.rust-lang.org/en-US/install.html)
    - Install `postgres`: `apt install postgresql libpq-dev`
    - Install [`migrant`](https://github.com/jaemk/migrant) (migration manager):
        - `cargo install migrant --features postgresql`
    - Initialize database (postgres):
        - `migrant init`
        - `migrant setup`
        - `migrant apply --all`
    - Build and run backend dev server:
        - `cargo run -- serve --port 3002`
        - Configuration can be tweaked in `config.ron`
    - Poke around in the database: `migrant shell`
- Frontend (inside `/web`):
    - Install [`npm`](https://www.npmjs.com/get-npm)
    - Install [`yarn`](https://yarnpkg.com/en/docs/install)
    - Build a run frontend dev server
        - `yarn start`
        - Open `http://localhost:3000`
        - Api requests are proxied to the backend: `localhost:3002`

Note: This project uses the `GitFlow` branching model, all pr's should be made to `develop`.


## Release Builds

To allow simple deployments, production/release artifacts are compiled and checked-in with tagged `releases` and `hotfixes`.
Both the backend (`rust`) and frontend (`react.js`) must be (re)compiled (only when code changes) for tagged commits (`releases` and `hotfixes`)

- Backend (`Rust` setup for cross-compilation)
    - Install [`docker`](https://www.digitalocean.com/community/tutorials/how-to-install-and-use-docker-on-ubuntu-16-04)
        - Add yourself to the `docker` group: `sudo usermod -a -G docker <user>`
        - Restart to pick up changes (logging in & out may suffice)
        - You should be able to run `docker version` without any errors
        - May need to start the Docker daemon if it's not already running: `sudo systemctl start docker` (not sure about windows/os-x)
    - Install [`cross`](https://github.com/japaric/cross): `cargo install cross`
    - Build server executables for targets listed in `build.py` script (currently only `x86_64`):
        - `build.py server`
- Frontend (`React`)
    - Build frontend app bundles and copy to their static-file locations
        - `build.py web`



## Deployment

> `postgres` & `nginx` are required

Note, the `master` branch is the release channel. All releases are tagged to allow easily jumping between versions.

- Initial
    - Clone this repo
    - `bin/x86_64/transfer admin database setup`
        - Run suggested commands to create database if it doesn't exist
        - Run `admin database setup` again
    - Make sure migrations are up to date: `bin/x86_64/transfer admin database migrate`
    - Poke around if you want: `bin/x86_64/transfer admin database shell`
    - Copy `nginx.conf.sample` to `/etc/nginx/sites-available/transfer` and update details
    - Copy `transfer.service.sample` to `/etc/lib/systemd/system/transfer.service` and update details
    - `systemctl restart nginx`
    - `systemctl restart transfer`
- Updates
    - `git pull --rebase=false`
    - `bin/x86_64/transfer admin database migrate`
    - `systemctl restart transfer`
- Selecting a specific version
    - `git fetch --all --tags`
    - `git checkout <tag>`
    - back to latest: `git checkout master`

