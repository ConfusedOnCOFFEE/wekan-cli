#!/bin/bash

docker-compose up -d
if [ "$1" == "ab" ]; then
    echo "docker build"
    ./e2e.sh b
    ./e2e.sh a
elif [ "$1" == "a" ]; then
    echo "Starting e2e tests: docker rm, run, logs and dc down"
    ./e2e.sh
    ./e2e.sh rm
    ./e2e.sh r
elif [ "$1" == "down" ]; then
    docker-compose down
elif [ "$1" == "rerun" ]; then
    ./e2e.sh b:tester
    ./e2e.sh r
elif [ "$1" == "b" ]; then
    ./e2e.sh b:wekan-cli
    ./e2e.sh b:tester
elif [ "$1" == "b:wekan-cli" ]; then
    cd ../../../
    docker build -f Dockerfile.e2e -t concafe/wekan-cli:integration .
elif [ "$1" == "b:tester" ]; then
    cd ../../../
    docker build --no-cache -f Dockerfile.e2e-retest -t concafe/wekan-cli:tester .
elif [ "$1" == "rm" ]; then
    docker stop wekan-cli
    docker rm wekan-cli
elif [ "$1" == "exec" ]; then
    docker run -it -e WEKAN_PWD=testuser123 \
           --network e2e_wekan-e2e-tier --name wekan-cli concafe/wekan-cli:tester "/bin/bash"
elif [ "$1" == "r" ]; then
    docker run -d --name wekan-cli --network e2e_wekan-e2e-tier concafe/wekan-cli:tester
fi
