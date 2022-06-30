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

```bash
SUCCESS STDOUT - LOGIN
SUCCESS STDERR - LOGIN
SUCCESS STDOUT - BOARD
SUCCESS STDERR - BOARD
SUCCESS STDOUT - LIST
SUCCESS STDERR - LIST
SUCCESS STDOUT - CARD
SUCCESS STDERR - CARD
SUCCESS STDOUT - DESCRIBE
SUCCESS STDERR - DESCRIBE
SUCCESS STDOUT - CONTEXT
SUCCESS STDERR - CONTEXT
SUCCESS STDOUT - DELETE
SUCCESS STDERR - DELETE
SUCCESS STDOUT - CONFIG
SUCCESS STDERR - CONFIG
```
