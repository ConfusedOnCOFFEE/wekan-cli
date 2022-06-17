# OBVIOUSLY DISCLAIMER

- PRIVATE PROJECT BUT OPEN TO USE IT
- BUILD AT YOUR OWN RISK, I try to assist. The CLI was tested on x86_64_linux, x86_64_MacOs and Apple ARM. With each new RC, I try to provide binaries.
- Message me if you found anything, which is criticial or against the rules.


# Wekan CLI


This projects aims to provide a CLI to view, create and update a WEKAN board, list and so on.
Also I try to learn RUST with this, so if you have tips or isuses, please create one.


```sh
wekan-cli 0.1.0
CLI to manage Wekan users, boards, lists, cards...

USAGE:
    wekan-cli [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -d, --no-store              Disable store for your wekan artifacts
    -f, --filter <FILTER>       Filter out available artifacts by id in format: b:..,l:..,c:.. Be
                                aware that this has a higher order then the name
    -h, --help                  Print help information
    -o, --format <FORMAT>       Output format: rust, elisp, long
    -q, --quiet                 Less output per occurrence
    -r, --no-recommendations    Disable next recommended workflow
    -v, --verbose               More output per occurrence
    -V, --version               Print version information

SUBCOMMANDS:
    board       Manage boards
    card        Manage tasks
    config      CLI configuration.
    describe    Describe artfifact by k8 syntax (type/name)
    get         List artifacts
    help        Print this message or the help of the given subcommand(s)
    inspect     Describe artifact by id
    list        Manage lists
    ps          Manage lists
    table       Show a board tree.
    
```


## Features

- Login via prompt, can be insecure as well or localhost.
- Set context to have multiple WEKAN hosts.
- Logout user, delete context.
- Show board, lists and cards.
- Remove board, list and cards.
- Update cards properties or move them to another list on the same board.
  - Move between lists.
  - Update due_at and end_at properties.
  - Change title, description and sort.
- Recommend your next workflow, after one command has been run.
- Store: Requests artifacts will be writen into the store locally. At the moment, this data can also be corrupted by the user.
  If the CLI doesn't find anything, it will NOT do a new REQUEST. In the future, it should do so. It can also be disabled with `-d`.
- `get` subcommand tries to parse your input type/name, like kubectl.
- `inspect` subcommand takes the original id. You can get the Id in the URL if you have a session open or if you use `-o long`.
- `table` subcommand tires to build a table, but it doesn't work yet.


## View

By default, we will show table style, highly inspired by Docker. The detail view also has date information.

```sh
ID    TITLE
1234  my_title
```


## Architecture

This repo contains three crates.

- wekan-cli uses clap to build a CLI with the help of derive API
- wekan-core uses mostly reqwest crate to do the heavy lifting of http requests.
- wekan-common keeps all the structs comming from the Wekan API and structs to build bodies for requests.

Motivation:

Not so sure, if that is a good way right now, but it could help if I upgrade to a new API version. Also wekan-core can be reused and not use it with the CLI.

## REFACTOR

- Currently there is not a good way on when a name argument is expected or not. I want to change that and make it more transparent and coherent in all subcommand but this takes time.


## ENV VARIABLES

### USAGE

- WEKAN_CLI_CONFIG_PATH sets the config_path. If you don't like $HOME/.config/wekan, you can change it.

### LOGGING

- WEKAN_LOG prints logging messages.
- WEKAN_BACKTRACE also printss external crates module messages.
- WEKAN_LOG_MFILTER allows to filter the logs, based on the modules.

All the ENV variables can be used in any combination, but MFILTER only filters non-third party modules.


## DEVELOPMENT

## TESTS

### E2E

I call E2E, the one, who are tested against an isolated environment, which cleans itself up after each run.

`./manager e2e` does exacly that. But first you need to create a user. Change directory to crates/wekan-cli/e2e and run `docker-compose up -d`. Visit `http://localhost:9999`and register a user. If you don't want to change everything, use testuser:testuser123.
If you have done this, you can run `./manager.sh e2e` and at the end, you can run `docker logs wekan-cli`. Hopefully, it will look like this:

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

## THANK YOU

Obviously, a lot of the usually research and thanks for the wonderful documentation from the RUST community, the rust book and especially the RUST compiler.  <3


## REFERENCES

- [WekanAPI v6.11](https://wekan.github.io/api/v6.11/#wekan-rest-api)
- [Wekan open-source Kanban](https://wekan.github.io/)
- [Github Wekan](https://github.com/wekan/wekan)


## LICENSE

Based on RUST, this project uses Apache and MIT license, please see the files [LICENSE-APACHE](./LICENSE-APACHE), [LICENSE-MIT](./LICENSE-MIT).
