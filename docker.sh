#!/bin/bash

set -ex

cmd="$1"
version="$(git rev-parse HEAD | awk '{ printf "%s", substr($0, 0, 7) }')"
reg="docker.jaemk.me"

if [ -z "$cmd" ]; then
    echo "missing command..."
    exit 1
elif [ "$cmd" = "build" ]; then
    docker build -t $reg/transfer:latest .
    if [ ! -z "$version" ]; then
        docker build -t $reg/transfer:$version .
    fi
elif [ "$cmd" = "push" ]; then
    $0 build
    docker push $reg/transfer:$version
    docker push $reg/transfer:latest
elif [ "$cmd" = "run" ]; then
    $0 build
    # hint, volume required: docker volume create transferdata
    docker run --rm --init -p 3300:3300 --env-file .env.docker --mount source=transferdata,destination=/transfer/uploads $reg/transfer:latest
elif [ "$cmd" = "shell" ]; then
    $0 build
    docker run --rm --init -p 3300:3300 --env-file .env.docker --mount source=transferdata,destination=/transfer/uploads -it $reg/transfer:latest /bin/bash
fi
