#!/bin/bash

set -ex

function main {
    ls web/package.json
    ls assets/static
    echo "** Clearing existing bundled files **"
    rm -f assets/static/js/main.js
    rm -f assets/static/css/main.css
    rm -f assets/static/media/*
    rm -f assets/static/manifest.json

    echo "** Building release bundles **"
    cd web
    yarn build

    echo "** Copying bundled files to static/assets **"
    cp build/static/js/main.*.js ../assets/static/js/main.js
    cp build/static/css/main.*.css ../assets/static/css/main.css
    cp build/static/media/* ../assets/static/media/
    cp public/manifest.json ../assets/static/manifest.json
}

main

