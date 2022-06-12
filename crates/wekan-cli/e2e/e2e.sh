#!/bin/bash

docker-compose up -d
set -e
if [ "$1" == "ab" ]; then
    echo "docker build"
    ./e2e.sh b
    ./e2e.sh a
elif [ "$1" == "a" ]; then
    echo "Starting e2e tests: docker rm, run, logs and dc down"
    ./e2e.sh
    ./e2e.sh rm
    ./e2e.sh r
    ./e2e.sh l
    ./e2e.sh down
elif [ "$1" == "down" ]; then
    docker-compose down
elif [ "$1" == "b" ]; then
    cd ../../../
    docker build -f Dockerfile.e2e -t concafe/wekan-cli:test .
elif [ "$1" == "rm" ]; then
    set +e
    docker stop wekan-cli
    docker rm wekan-cli
    set -e
elif [ "$1" == "l" ]; then
    docker logs wekan-cli
elif [ "$1" == "exec" ]; then
    docker run -it -e WEKAN_PWD=testuser123 \
           --network e2e_wekan-e2e-tier --name wekan-cli concafe/wekan-cli:test "/bin/bash"
elif [ "$1" == "r" ]; then
    docker run -d --name wekan-cli --network e2e_wekan-e2e-tier \
       -e WEKAN_URL=wekan-e2e-app:8080 --entrypoint "e2e/entrypoint.sh" \
       -e WEKAN_USER=testuser -e WEKAN_PWD=testuser123 concafe/wekan-cli:test
fi
