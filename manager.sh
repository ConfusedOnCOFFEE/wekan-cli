#!/bin/bash
cmd=$1
crates/wekan-cli/e2e/e2e.sh rm >/dev/null 2>/dev/null
if [ "${cmd}" == "d" ]; then
    cd crates/wekan-cli
    export EMACSSAVEMODEDIR=.
    emacs
elif [ "${cmd}" == "test" ]; then
     cd crates/wekan-cli
     cargo test -- --nocapture
elif [ "${cmd}" == "e2e" ]; then
    cd crates/wekan-cli/e2e
    ./e2e.sh ab
elif [ "${cmd}" == "e2e:rerun" ]; then
    cd crates/wekan-cli/e2e
    ./e2e.sh rerun
elif [ "${cmd}" == "docker:build" ]; then
    docker build -t concafe/wekan-cli:release .
elif [ "${cmd}" == "run" ]; then
    docker run -d --name wekan-cli --network e2e_wekan-e2e-tier concafe/wekan-cli:release /bin/bash
elif [ "${cmd}" == "qa" ]; then
    set -e
    ./manager.sh clippy
    ./manager.sh fmt
elif [ "${cmd}" == "clippy" ]; then
    ./manager.sh clippy:cli
    ./manager.sh clippy:core
    ./manager.sh clippy:common
    ./manager.sh clippy:macro
elif [ "${cmd}" == "clippy:cli" ]; then
    echo "wekan-cli clippy"
    cd crates/wekan-cli
    cargo clippy -- -Dwarnings
elif [ "${cmd}" == "clippy:core" ]; then
    echo "wekan-core clippy"
    cd crates/wekan-core
    cargo clippy -- -Dwarnings
elif [ "${cmd}" == "clippy:common" ]; then
    echo "wekan-common clippy"
    cd crates/wekan-common
    cargo clippy -- -Dwarnings
elif [ "${cmd}" == "clippy:macro" ]; then
    echo "wekan-core-derive clippy"
    cd crates/wekan-core-derive
    cargo clippy -- -Dwarnings
elif [ "${cmd}" == "fmt" ]; then
    echo "fmt crates"
    cd crates/wekan-cli
    cargo fmt
    cd ../wekan-core
    cargo fmt
    cd ../wekan-core-derive
    cargo fmt
    cd ../wekan-common
    cargo fmt
elif [ "${cmd}" == "test" ]; then
    cd crates/wekan-cli
    cargo test
    cd ../crates/wekan-core
    cargo test
elif [ "${cmd}" == "build" ]; then
    cd crates/wekan-cli
    cargo build --verbose
elif [ "${cmd}" == "release:apple" ]; then
    cd crates/wekan-cli
    cargo build -r --target aarch64-apple-darwin
    cargo build -r --target x86_64-apple-darwin
elif [ "${cmd}" == "release:linux" ]; then
    cd crates/wekan-cli
    cargo build -r --target x86_64-unknown-linux-gnu
elif [ "${cmd}" == "release:windows" ]; then
    cd crates/wekan-cli
    cargo build -r --target x86_64-pc-windows-gnu
elif [ "${cmd}" == "e2e:rerun" ]; then
    cd crates/wekan-cli/e2e
    ./e2e.sh rm
    ./e2e.sh r
    ./e2e.sh l
fi
