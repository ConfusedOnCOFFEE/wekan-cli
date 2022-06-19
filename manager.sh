#!/bin/bash
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
flow=$1
selection=$2
all_args="${@:2}"

# Spin up testing environment.
echo "Quiet e2e docker-compose command."
crates/wekan-cli/e2e/e2e.sh rm >/dev/null 2>/dev/null

# Run tests with different crates including the available features.
function test_crates() {
    echo "Run test ${1}"
    cd $script_dir
    if [ "${1}" == "cli" ]; then
        cd crates/wekan-cli
        cargo test
        cargo test --features store
    elif [ "${1}" == "core" ]; then
        cd crates/wekan-core
        cargo test
        cargo test  --features store
    else
        test_crates cli
        test_crates core
    fi
}

# Run E2E tests and show results.
function e2e() {
    echo "Run e2e ${1}"
    cd $script_dir
    if [ "${1}" == "ab" ]; then
        cd crates/wekan-cli
        cargo build --features integration
        cd ./e2e
        ./e2e.sh ab
    elif [ "${1}" == "c" ]; then
        e2e ab
        e2e rerun
    elif [ "${1}" == "rerun" ]; then
        cd crates/wekan-cli/e2e
        ./e2e.sh rerun
        echo "Sleeping 5 seconds so the test can run."
        sleep 5
        echo "Trying to present results:"
        docker logs wekan-cli
    elif [ "${1}" == "l" ]; then
        docker logs wekan-cli
    else
        e2e ab
    fi
}

# Clippy all crates
function clippy() {
    echo "Run clippy ${1}"
    cd $script_dir
    if [ "${1}" == "cli" ]; then
        cd crates/wekan-cli
        cargo clippy -- -Dwarnings
        cargo clippy --features store -- -Dwarnings
    elif [ "${1}" == "core" ]; then
        cd crates/wekan-core
        cargo clippy -- -Dwarnings
        cargo clippy --features store -- -Dwarnings
    elif [ "${1}" == "common" ]; then
        cd crates/wekan-common
        cargo clippy -- -Dwarnings
    elif [ "${1}" == "macro" ]; then
        cd crates/wekan-core-derive
        cargo clippy -- -Dwarnings
    else
        clippy cli
        clippy core
        clippy common
        clippy macro
    fi
}


# Cmt all crates
function fmt() {
    echo "fmt crates"
    cd $script_dir
    cd crates/wekan-cli
    cargo fmt
    cd $script_dir
    cd crates/wekan-core
    cargo fmt
    cd $script_dir
    cd crates/wekan-core-derive
    cargo fmt
    cd $script_dir
    cd crates/wekan-common
    cargo fmt
}


# Build release artifact for specified platforms.
function release() {
    echo "Release ${1}"
    cd $script_dir
    if [ "${1}" == "apple" ]; then
        cd crates/wekan-cli
        cargo build -r --target aarch64-apple-darwin
        cargo build -r --target x86_64-apple-darwin
    elif [ "${1}" == "linux" ]; then
        cd crates/wekan-cli
        cargo build -r --target x86_64-unknown-linux-gnu
    elif [ "${1}" == "windows" ]; then
        cd crates/wekan-cli
        cargo build -r --target x86_64-pc-windows-gnu
    else
        cd crates/wekan-cli
        cargo build -r
    fi
}


# Run wekan-cli with cargo run.
function run() {
    cd $script_dir
    if [ "${1}" == "cli" ]; then
        echo "Run: ${1} with ${all_args}"
        cd crates/wekan-cli
        cargo run -- ${all_args}
    elif [ "${1}" == "cli-store" ]; then
        echo "Run: ${1} with ${all_args}"
        cd crates/wekan-cli
        cargo run --features store -- ${all_args}
    elif [ "${1}" == "container" ]; then
        docker run -d --name wekan-cli --network e2e_wekan-e2e-tier concafe/wekan-cli:release /bin/bash
    else
        run cli-store
    fi
}


# Decide which flow to run.
case $flow in
    "d"|"dev")
        cd crates/wekan-cli
        export EMACSSAVEMODEDIR=.
        emacs
        exit
        ;;
    "t"|"test")
        test_crates $selection
        exit
        ;;
    "b")
        cd crates/wekan-cli
        cargo build --verbose
        exit
        ;;
    "d:b")
        docker build -t concafe/wekan-cli:release .
        exit
        ;;
    "f"|"fmt")
        fmt
        exit
        ;;
    "c"|"clippy")
        clippy $selection
        exit
        ;;
    "qa")
        ./manager.sh fmt
        ./manager.sh clippy
        ./manager.sh t
        exit
        ;;
    "release")
        release $selection
        exit
        ;;
    "r"|"run")
        run $selection
        exit
        ;;
    "e"|"e2e"|"2e"|"2ee")
        e2e $selection
        exit
        ;;
    *)
        echo -e "Nothing selected."
        exit
        ;;
esac
echo "Flow done."
