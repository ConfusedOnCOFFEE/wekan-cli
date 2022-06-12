#!/bin/bash
crates/wekan-cli/e2e/e2e.sh rm
if [ "${cmd}" == "d" ]; then
    cd crates/wekan-cli
    export EMACSSAVEMODEDIR=.
    emacs
elif [ "${cmd}" == "e2e" ]; then
    cd crates/wekan-cli/e2e
    ./e2e.sh ab
elif [ "${cmd}" == "docker:build" ]; then
    docker build -t concafe/wekan-cli:release .
elif [ "${cmd}" == "run" ]; then
    docker run -d --name wekan-cli --network e2e_wekan-e2e-tier concafe/wekan-cli:release /bin/bash
elif [ "${cmd}" == "e2e:rerun" ]; then
    cd crates/wekan-cli/e2e
    ./e2e.sh rm
    ./e2e.sh r
    ./e2e.sh l
fi
