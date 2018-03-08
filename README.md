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
        - `yarn install`
        - `yarn start`
        - Open `http://localhost:3000`
        - Api requests are proxied to the backend: `localhost:3002`


## Release Builds

Packaged releases are built and packaged by travis-ci. Complete packaged releases are available [here](https://github.com/jaemk/transfer/releases)

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


## Deployment / Running Packaged Releases

> `postgres` & `nginx` are required

- **Initial Setup**
    - Create and enter a project directory where versioned packages can be managed:
      ```bash
      mkdir transfer
      cd transfer
      ```
    - Download, unpackage, and do initial setup for the latest release
      (see [releases](https://github.com/jaemk/transfer/releases))
       ```bash
       # download
       curl -LO https://github.com/jaemk/transfer/releases/download/$TAG/transfer-$TAG-$TARGET.tar.gz
       # extract
       tar -xf transfer-$TAG-$TARGET.tar.gz
       # rename
       mv transfer $TAG
       # setup "latest" symlink
       ln -sfn $TAG latest
       ```
    - Setup an uploads directory where transfer uploads can exist between application code updates.
      Make sure your `config.ron` file is updated and copied to the config directory.
      ```bash
      mkdir transfer_uploads
      vim latest/config.ron  # update "upload_directory" to "/<ABS_PATH_TO>/transfer/transfer_uploads"
      # and copy to the config directory
      cp latest/config.ron `latest/bin/transfer admin config-dir`
      ```
    - Setup the database
      ```bash
      latest/bin/transfer admin database setup
      # Run suggested commands to create database if it doesn't exist
      # and then try settinng up migrations again
      latest/bin/transfer admin database setup
      ```
    - Apply migrations
      ```bash
      latest/bin/transfer admin database migrate
      ```
    - Poke around the database
      ```bash
      bin/x86_64/transfer admin database shell
      ```
    - Setup nginx
      ```bash
      # copy sample config and then update its details with your environment info
      sudo cp nginx.conf.sample /etc/nginx/sites-available/transfer
      # check config
      sudo nginx -t
      # enable site
      sudo ln -s /etc/nginx/sites-available/transfer /etc/nginx/sites-enabled/transfer
      sudo systemctl restart nginx
      ```
    - Setup systemd service
      ```bash
      # copy sample config and then update its details with your environment info
      sudo cp transfer.service.sample /lib/systemd/system/transfer.service
      # enable the service
      sudo systemctl daemon-reload
      sudo systemctl enable transfer.service
      # start!
      sudo systemctl restart transfer
      # tail the log
      sudo journalctl -fu transfer
      ```
- **Updates**
    - Assuming you followed the "Initial Setup" section
    - Use the `release.py` script to fetch, unpackage, and symlink the latest release
      ```bash
      # from the `transfer` project root
      # follow prompts to download the appropriate target and replace the `latest` symlink
      latest/release.py fetch
      ```
    - Apply migrations and restart the app
      ```bash
      bin/x86_64/transfer admin database migrate
      systemctl restart transfer
      ```
