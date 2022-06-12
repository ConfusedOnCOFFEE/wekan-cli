#!/bin/bash
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
exec 2>&1
# exec >/log/stdout.log
# exec 2>/log/stderr.log
# if [ "$1" == "ld" ]; then
#     cd crates/wekan-cli
#     cargo build --features cache >/dev/null 2>&1
#     cd target/debug
# elif [ "$1" == "lr" ]; then
#     cd crates/wekan-cli
#     cargo build -r --features cache >/dev/null 2>&1
#     cd ../target/release
# fi
script_home="$( cd "$( dirname "${BASH_SOURCE[0]}" ) " > /dev/null 2>&1 && pwd )"
result() {
    if diff test_stdout $2; then
        printf "STDOUT: ${GREEN}${1} successful.\n${NC}"
    else
        printf "STDERR: ${RED}ERROR: ${1} failed.\n${NC}"
    fi
    if diff test_stderr stderr_all; then
        printf "STDOUT: ${GREEN}${1} successful.\n${NC}"
    else
        printf "STDERR: ${RED}ERROR: ${1} failed.\n"
    fi
    rm test_stderr
    rm test_stdout
}

board() {
    for cmd_to_run in 'wekan-cli board --help' 'wekan-cli board create Test' 'wekan-cli board ls' 'wekan-cli board Test'
    do
        $cmd_to_run >>test_stdout 2>>test_stderr
    done
}

list() {
   for cmd_to_run in 'wekan-cli list --help' 'wekan-cli list -b Test ls' 'wekan-cli list -b Test create Test' 'wekan-cli list -b Test ls' 'wekan-cli board Test' 'wekan-cli board Test details'
   do
       $cmd_to_run >>test_stdout 2>>test_stderr
   done
}

card() {
    for cmd_to_run in 'wekan-cli card --help' 'wekan-cli card -b Test -l Test create -d description test-card' 'wekan-cli card -b Test -l Test test-card' 'wekan-cli list -b Test Test details' 'wekan-cli card -b Test -l Test test-card details'
    do
        $cmd_to_run >>test_stdout 2>>test_stderr
    done
}

cleanup() {
    for cmd_to_run in 'wekan-cli -d card -b Test -l Test rm test-card' 'wekan-cli -d list -b Test Test rm' 'wekan-cli -d board Test rm'
    do
        $cmd_to_run >>test_stdout 2>>test_stderr
    done
}

cd e2e
wekan-cli config set-credentials -i --domain-and-port $WEKAN_URL $WEKAN_USER >test_stdout 2>test_stderr
result Login stdout_login
board
result BOARD stdout_board
list
result LIST stdout_list
card
result CARD stdout_card
cleanup
result DELETE stdout_delete
