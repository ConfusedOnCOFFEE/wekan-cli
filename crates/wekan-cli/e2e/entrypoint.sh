#!/bin/bash
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
exec 2>&1
script_home="$( cd "$( dirname "${BASH_SOURCE[0]}" ) " > /dev/null 2>&1 && pwd )"
result() {
    if diff test_stdout $2; then
        printf "STDOUT: ${GREEN}${1} success.\n${NC}"
    else
        printf "STDOUT: ${RED}ERROR: ${1} failed.\n${NC}"
    fi
    if diff test_stderr $3; then
        printf "STDERR: ${GREEN}${1} success.\n${NC}"
    else
        printf "STDERR: ${RED}ERROR: ${1} failed.\n${NC}"
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

describe() {
    for cmd_to_run in 'wekan-cli describe board/Test'
    do
        $cmd_to_run >>test_stdout 2>>test_stderr
    done
}

context() {
    for cmd_to_run in 'wekan-cli config --help' 'wekan-cli config set-context local' 'wekan-cli config use-context local' 'wekan-cli config remove --help'
    do
        $cmd_to_run >>test_stdout 2>>test_stderr
    done
}

config() {
    for cmd_to_run in 'wekan-cli config delete-context local'  'wekan-cli config delete-credentials' 'wekan-cli config remove please' 'wekan-cli config remove -c local -y please' 'wekan-cli config remove -c local -f -y please' 'wekan-cli config remove -y please'
    do
        $cmd_to_run >>test_stdout 2>>test_stderr
    done
}

cd e2e
wekan-cli board ls >test_stdout 2>test_stderr # expected to fail
wekan-cli config set-credentials -i --host $WEKAN_URL $WEKAN_USER >>test_stdout 2>>test_stderr
result Login stdout_login stderr_pre_login
board
result BOARD stdout_board stderr_all
list
result LIST stdout_list stderr_all
card
result CARD stdout_card stderr_all
describe
result DESCRIBE stdout_describe stderr_all
context
result CONTEXT stdout_context stderr_all
cleanup
result DELETE stdout_delete stderr_all
config
result CONFIG stdout_config stderr_config
