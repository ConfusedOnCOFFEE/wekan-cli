#!/bin/bash

# This script allows to run E2E tests and communicate with containers and cargo.
# Comfortable rerun, build and inspect the result of a test run.
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
echo "Starting e2e docker-compose container in detached mode"
docker-compose up -d

function e2e() {
    case "$1" in
        "ab")
            e2e b:wekan-cli
            e2e rerun
            ;;
        "down")
            docker-compose down
            ;;
        "rerun")
            e2e b:tester
            e2e rm
            e2e r
            ;;
        "b:wekan-cli")
            echo "docker build wekan-cli"
            cd $script_dir/../
            docker build --no-cache -f Dockerfile.e2e -t concafe/wekan-cli:integration .
            ;;
        "b:tester")
            echo "docker build tester"
            cd $script_dir/../
            docker build --no-cache -f Dockerfile.e2e-retest -t concafe/wekan-cli:tester .
            ;;
        "rm")
            docker stop wekan-cli
            docker rm wekan-cli
            ;;
        "exec")
            docker run -it -e WEKAN_PWD=testuser123 \
                   --network e2e_wekan-e2e-tier --name wekan-cli concafe/wekan-cli:tester "/bin/bash"
            ;;
        "r")
            echo "docker run tester"
            docker run --name wekan-cli --network e2e_wekan-e2e-tier concafe/wekan-cli:tester
            ;;
    esac
}

e2e $1
