# E2E

Currently E2E tests only exist for the wekan-cli binary.

## Prerequisites

- Docker
- docker-compose


## Setup


Let's crate a user. Change directory to crates/wekan-cli/e2e and run `docker-compose up -d`.
Visit `http://localhost:9999`and register a user. If you don't want to change everything, use testuser:testuser123.


## Test

Run `./manager.sh e2e c` to start WEKAN containers, build the wekan-cli container.
Inside the container, the `entrypoint.sh` script will run the test cases and compare them.
A test run without errors should look like this:

```sh
STDOUT: Login success.
STDERR: Login success.
STDOUT: BOARD success.
STDERR: BOARD success.
STDOUT: LIST success.
STDERR: LIST success.
STDOUT: CARD success.
STDERR: CARD success.
STDOUT: DESCRIBE success.
STDERR: DESCRIBE success.
STDOUT: CONTEXT success.
STDERR: CONTEXT success.
STDOUT: DELETE success.
STDERR: DELETE success.
STDOUT: CONFIG success.
STDERR: CONFIG success
```
