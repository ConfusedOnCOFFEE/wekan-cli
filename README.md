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


# Features

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


# View

By default, we will show table style, highly inspired by Docker. The detail view also has date information.

```sh
ID    TITLE
1234  my_title
```


# ARCHITECTURE

Please take a look in [DEVELOPMENT](./DEVELOPMENT.md).


# INSTALL

Download the binary from the release page or clone the repository and build it yourself.
Afterwards you get binary in crates/wekan-cli/target/release/${platform}/wekan-cli to use.

# BUILD 


- Clone the repo
- `cd wekan-cli/crates/wekan-cli` in the directory
- Run:
- - `cargo build -r --features store` and move it where you want ti.
- - `cargo install --features store` and use it.

# ENV VARIABLES

The CLI places everything inside WEKAN_CLI_CONFIG_PATH.

## USAGE

- WEKAN_CLI_CONFIG_PATH sets the config_path. If you don't like $HOME/.config/wekan, you can change it.

## LOGGING

- WEKAN_LOG prints logging messages.
- WEKAN_BACKTRACE also printss external crates module messages.
- WEKAN_LOG_MFILTER allows to filter the logs, based on the modules.

All the ENV variables can be used in any combination, but MFILTER only filters wekan-* modules.


## DEVELOPMENT

[DEVELOPMENt](.DEVELOPMENT.md)

## TESTS

## Unittests

`cargo run test` for wekan-cli and wekan-core. WekanCommon doesn't need tests.


### E2E

[E2E](./E2E.md)

### Coverage

[COVERAGE](./COVERAGE.md)


## THANK YOU

Obviously, a lot of the usually research and thanks for the wonderful documentation from the RUST community, the rust book and especially the RUST compiler.  <3


# REFERENCES

- [WekanAPI v6.11](https://wekan.github.io/api/v6.11/#wekan-rest-api)
- [Wekan open-source Kanban](https://wekan.github.io/)
- [Github Wekan](https://github.com/wekan/wekan)


# LICENSE

Based on RUST and different crates. This project uses Apache and MIT license, please see the files [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT).
