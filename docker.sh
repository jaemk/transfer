#!bin/bash

set -ex

cmd="$1"
version="$2"

if [ -z "$cmd" ]; then
    echo "missing command..."
    exit 1
elif [ "$cmd" = "build" ]; then
    docker build -t jaemk/transfer:latest .
    if [ ! -z "$version" ]; then
        docker build -t jaemk/transfer:$version .
    fi
elif [ "$cmd" = "run" ]; then
    # hint, volume required: docker volume create transferdata
    docker run --rm --init -p 3300:3300 --env-file .env.docker --mount source=transferdata,destination=/transfer/uploads jaemk/transfer:latest
elif [ "$cmd" = "shell" ]; then
    docker run --rm --init -p 3300:3300 --env-file .env.docker --mount source=transferdata,destination=/transfer/uploads jaemk/transfer:latest /bin/bash
fi
