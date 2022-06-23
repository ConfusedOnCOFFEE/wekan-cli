#!/bin/bash
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
script_home="$( cd "$( dirname "${BASH_SOURCE[0]}" ) " > /dev/null 2>&1 && pwd )"
sleep 5
exec 2>&1
success=SUCCESS
failed=FAILED

result() {
    diff_channels $1 $2 $3
    rm test_stderr
    rm test_stdout
}

diff_channels() {
    std_out_msg=`echo "stdout - ${1}" | tr '[:lower:]' '[:upper:]'`
    std_err_msg=`echo "stderr - ${1}" | tr '[:lower:]' '[:upper:]'`
    diff_and_print_result "$std_out_msg" test_stdout $2
    diff_and_print_result "$std_err_msg" test_stderr $3
}

diff_and_print_result() {
    if diff $2 $3; then
        printf "${GREEN}$success ${NC}$1\n"
    else
        printf "${RED}$failed ${NC}$1\n"
    fi
}

execute_test() {
    echo "-------------------    $1    -------------------" >>test_stdout
    $1 >>test_stdout 2>>test_stderr
}

board() {
    for cmd_to_run in 'wekan-cli board --help' 'wekan-cli board create Test' 'wekan-cli board ls' 'wekan-cli board -n Test' 'wekan-cli board -n Test details'
    do
        execute_test "$cmd_to_run"
    done
}

list() {
    for cmd_to_run in 'wekan-cli list --help' 'wekan-cli list -b Test ls' 'wekan-cli list -b Test create Test' 'wekan-cli list -b Test ls' 'wekan-cli list -b Test -n Test details' 'wekan-cli board -n Test' 'wekan-cli board -n Test details'
    do
        execute_test "$cmd_to_run"
    done
}

card() {
    for cmd_to_run in 'wekan-cli card --help' 'wekan-cli -d card -b Test -l Test create -d description test-card' 'wekan-cli -d card -b Test -l Test -n test-card' 'wekan-cli -d list -b Test -n Test details' 'wekan-cli -d card -b Test -l Test -n test-card details'
    do
        execute_test "$cmd_to_run"
    done
}

delete() {
    for cmd_to_run in 'wekan-cli -d card -b Test -l Test -n test-card rm' 'wekan-cli -d list -b Test -n Test rm' 'wekan-cli -d board -n Test rm'
    do
        execute_test "$cmd_to_run"
    done
}

describe() {
    for cmd_to_run in 'wekan-cli describe board/Test'
    do
        execute_test "$cmd_to_run"
    done
}

context() {
    for cmd_to_run in 'wekan-cli config --help' 'wekan-cli config set-context local' 'wekan-cli config use-context local'
    do
        execute_test "$cmd_to_run"
    done
}

config() {
    for cmd_to_run in 'wekan-cli config remove-context local'  'wekan-cli config remove-credentials' 'wekan-cli config remove please' 'wekan-cli config remove -c local -y please' 'wekan-cli config remove -c local -f -y please' 'wekan-cli config remove -y please'
    do
        execute_test "$cmd_to_run"
    done
}

test_runner() {
    $1
    result "$1" stdout_$1 stderr_all
}

sleep 2
wekan-cli board ls >test_stdout 2>test_stderr # expected to fail
wekan-cli config set-credentials -i --host $WEKAN_URL $WEKAN_USER >>test_stdout 2>>test_stderr

result LOGIN stdout_login stderr_pre_login
test_runner board
test_runner list
test_runner card
test_runner describe
test_runner context
test_runner delete
config
result CONFIG stdout_config stderr_config
