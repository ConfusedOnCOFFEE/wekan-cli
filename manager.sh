#!/bin/bash
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
flow=$1
selection=$2
all_args="${@:2}"
os_type=$(uname)

# Run tests with different crates including the available features.
function test_crates() {
    echo "Run test ${1}"
    cd $script_dir
    case "$1" in
        "cli")
            run_test wekan-cli
            ;;
        "cli-store")
            cd crates/wekan-cli
            cargo test --features store -- --nocapture
            ;;
        "core")
            run_test wekan-core
            cargo test --features store -- --nocapture
            ;;
        *)
            cargo test
            cargo test --features store -- --nocapture
            ;;
    esac
}

function run_test() {
    cd crates/$1
    cargo test -- --nocapture
}

# Run E2E tests and show results.
function e2e() {
    echo "Run e2e $1"
    cd $script_dir
    case "$1" in
        "c")
            run_e2e
            e2e rerun
            ;;
        "rerun"|"r")
            cd e2e
            ./e2e.sh rerun
            ;;
        "l")
            docker logs wekan-cli
            ;;
        *)
            run_e2e
            ;;
    esac
}

function run_e2e() {
    e2e/e2e.sh rm >/dev/null 2>/dev/null
    cargo build --features integration
    cd e2e
    ./e2e.sh ab
}


# Clippy all crates
function clippy() {
    echo "Run clippy ${1}"
    cd $script_dir
    case "$1" in
        "cli")
            run_clippy wekan-cli
            cargo clippy --features store -- -Dwarnings
            ;;
        "core")
            run_clippy wekan-core
            cargo clippy --features store -- -Dwarnings
            ;;
        "common")
            run_clippy wekan-common
            ;;
        "macro")
            clippy wekan-core-derive
            clippy wekan-cli-derive
            ;;
        *)
            cargo clippy -- -Dwarnings
            ;;
    esac
}

function run_clippy() {
    cd crates/$1
    cargo clippy -- -Dwarnings
}


# Cmt all crates
function fmt() {
    echo "fmt crates"
    cargo fmt
}


# Build release artifact for specified platforms.
function release() {
    echo "Release ${1}"
    cd $script_dir
    case "$1" in
        "apple")
            cd crates/wekan-cli
            cargo build -r --target aarch64-apple-darwin
            cargo build -r --target x86_64-apple-darwin
            ;;
        "linux")
            cd crates/wekan-cli
            cargo build -r --target x86_64-unknown-linux-gnu
            ;;
        "windows")
            cd crates/wekan-cli
            cargo build -r --target x86_64-pc-windows-gnu
            ;;
        *)
            cd crates/wekan-cli
            cargo build -r
            ;;
    esac
}


# Run wekan-cli with cargo run.
function run() {
    cd $script_dir
    case "$1" in
        "cli")
            echo "Run: $1 with $all_args"
            cargo run -- ${all_args}
            ;;
        "cli-store")
            echo "Run: $1 with $all_args"
            cargo run --features store -- $all_args
            ;;
        "container")
            docker run -d --name wekan-cli --network e2e_wekan-e2e-tier concafe/wekan-cli:release /bin/bash
            exit $?
            ;;
        *)
            run cli-store
            ;;
    esac
}


# RECOMMENDED
# https://github.com/mozilla/grcov
function mozilla_gcov() {
    grcov_exist="$(grcov &>/dev/null)"
    if [ "$?" != "1" ]; then
        echo "Install grcov first with 'cargo install grcov'"
        exit 1
    fi
    cd crates/wekan-cli
    case "$1" in
        "gen")
            grcov . -s .  --binary-path ./target/debug/ -t html \
                  --branch --ignore-not-existing -o ./target/debug/coverage/
            delete_llvm_profile
            if [ "$?" == "0" ]; then
                case "$os_type" in
                    "Darwin")
                        # Opening with Safari as default choice. use Please Firefox :)
                        open "$script_dir/crates/wekan-cli/target/debug/coverage/index.html"
                        ;;
                    "*")
                        echo "Open $script_dir/crates/wekan-cli/target/debug/coverage/index.html"
                        ;;
                esac
            fi
            ;;
        "rm")
            rm_llvm_profiles
            ;;
        *)
            echo "Make sure, to clean the ENV variables after the run, so you don't have sidefects."
            echo "Please run these commands in the root directory:"
            export_flag_cargo_build='echo "Set RUSTFLAGS" && export RUSTFLAGS="-Cinstrument-coverage"'
            change_directory='cd crates/wekan-cli'
            cargo_build='cargo build --features store'
            export_flag_llvm_profile='echo "Set LLVM_PROFILE_FILE" && export LLVM_PROFILE_FILE="llvm-profile-%p-%m.profraw"'
            cargo_test='cargo test --features store'
            export_clear='export RUSTFLAGS= && export LLVM_PROFILE_FILE='
            echo -e "$export_flag_cargo_build && \n $change_directory && \n $cargo_build && \n $export_flag_llvm_profile && \n $cargo_test && \n cd ../../ && ./manager.sh cov gen && \n echo 'Clear ENV variables' && \n $export_clear"
            ;;
    esac
}

function rm_llvm_profiles () {
    echo "Cleaning up generated files"
    cd $script_dir
    rm default.profraw 2>/dev/null
    rm llvm-profile*.profraw 2>/dev/null
    cd crates/wekan-cli
    rm default.profraw 2>/dev/null
    rm llvm-profile*.profraw 2>/dev/null
    cd "$script_dir/crates/wekan-core"
    rm default.profraw 2>/dev/null
    rm llvm-profile*.profraw 2>/dev/null
    cd "$script_dir/crates/wekan-common"
    rm default.profraw 2>/dev/null
    rm llvm-profile*.profraw 2>/dev/null
}

# ------ OUTDATED OR NOT WORKING ----
# Not working!!
# https://users.rust-lang.org/t/howto-generating-a-branch-coverage-report/8524
function lcov_coverage() {
    cd crates/wekan-cli
    os_type=$(uname)
    case "$os_type" in
        "Darwin")
            {
                cargo +nightly rustc --bin wekan-cli -- \
                      --test \
                      -Ccodegen-units=1 \
                      -Clink-dead-code \
                      -Cpasses=insert-gcov-profiling \
                      -Zno-landing-pads \
                      -L/Library/Developer/CommandLineTools/usr/lib/clang/8.1.0/lib/darwin/ \
                      -lclang_rt.profile_osx
            } ;;
        "Linux")
            {
                cargo +nightly rustc  --bin wekan-cli -- --test \
                      -Ccodegen-units=1 \
                      -Clink-dead-code \
                      -Cpasses=insert-gcov-profiling \
                      -Zno-landing-pads \
                      -L/usr/lib/llvm-3.8/lib/clang/3.8.1/lib/linux/ \
                      -lclang_rt.profile-x86_64
            } ;;
        *)
            {
                echo "Unsupported OS, exiting"
                exit
            } ;;
    esac

    LCOVOPTS="--gcov-tool llvm-gcov --rc lcov_branch_coverage=1"
    LCOVOPTS="${LCOVOPTS} --rc lcov_excl_line=assert"
    lcov ${LCOVOPTS} --capture --directory . --base-directory . \
         -o target/coverage/raw.lcov
    lcov ${LCOVOPTS} --extract target/coverage/raw.lcov "$(pwd)/*" \
         -o target/coverage/raw_crate.lcov
    genhtml --branch-coverage --demangle-cpp --legend \
            -o target/coverage/ target/coverage/raw_crate.lcov
}

# Decide which flow to run.
case $flow in
    "b"|"build")
        echo "Build without feature"
        cargo build
        echo "Build feture store"
        cargo build --features store
        echo "Build integration"
        cargo build --features integration
        ;;
    "b:target")
        release $selection
        ;;
    "c"|"clippy")
        clippy $selection
        ;;
    "cov"|"lcov")
        mozilla_gcov $selection
        ;;
    "d"|"dev")
        export EMACSSAVEMODEDIR=.
        emacs
        ;;
    "d:b")
        docker build -t concafe/wekan-cli:release .
        ;;
    "e"|"e2e"|"2e"|"2ee")
        e2e $selection
        ;;
    "f"|"fmt")
        fmt
        ;;
    "r"|"run")
        run $selection
        ;;
    "r:s")
        run cli-store
        ;;
    "t"|"test")
        test_crates $selection
        ;;
    "ts")
        test_crates cli-store
        ;;
    "qa")
        fmt
        clippy
        test_crates
        e2e
        ;;
    *)
        echo "Nothing selected"
        ;;
esac
exit $?
