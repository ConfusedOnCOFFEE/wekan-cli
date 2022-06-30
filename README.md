# OBVIOUSLY DISCLAIMER

- PRIVATE PROJECT BUT READY TO BE USED AND ADAPTED
- BUILD AT YOUR OWN RISK, I try to assist. The CLI was tested on x86_64_linux, x86_64_MacOs and Apple ARM. 
- Message me if you find anything, which is criticial or against the rules.
- I don't provide releases, maybe some tags if a new feature was added but HEAD will always be stable.


# Wekan CLI


This projects aims to provide a CLI to view, create and update a WEKAN board, list and so on.
Also I try to learn RUST with this, so if you have tips or isuses, please create one.


```bash
OPTIONS:
    -d, --no-store                         Disable store for your wekan artifacts
    -f, --filter <FILTER>                  Filter out artifacts by id
    -h, --help                             Print help information
    -o, --output-format <OUTPUT_FORMAT>    Output format: rust, elisp, long, extended
    -q, --quiet                            Less output per occurrence
    -r, --no-recommendations               Disable next recommended workflow
    -v, --verbose                          More output per occurrence
    -V, --version                          Print version information

SUBCOMMANDS:
    apply        Apply a change to an artifact
    board        Manage boards
    card         Manage tasks
    checklist    Manage checklists
    config       CLI configuration
    describe     Describe artfifact
    get          Get an artifact
    help         Print this message or the help of the given subcommand(s)
    inspect      Describe artifact by id
    list         Manage lists
    table        Show a board table
```


# Features

- Login via prompt, can be insecure as well or localhost
- Set context to have multiple WEKAN hosts
- Logout user and remove contexts
- Show board, lists, cards and checklists
- Remove board, list, cards and checklists
- Update cards properties:
  - Move between lists of the same board
  - Update title, description, due_at, end_at and sort properties
- Recommend your next workflow, after one command has been run
- Store: Requests artifacts will be writen into the store locally. At the moment, this data can also be corrupted by the user.
  If the CLI doesn't find anything, it will do a new request. Using of local store can also be disabled with `-d`.
- `get` subcommand tries to parse your input type/name, like kubectl
- `inspect` subcommand takes the original id. You can get the Id in the URL if you have a session open or if you use `-o ext`.
- `table` build are table of one board, where all lists and cards are arrange in the same order as on the webpage


# View

By default, we will show table style, highly inspired by Docker. The detail view also has date information and more.

```bash
ID    TITLE
1234  my_title
```


# ARCHITECTURE

Please take a look in [DEVELOPMENT](./doc/DEVELOPMENT.md).


# INSTALL

Download the binary from the release page or clone the repository and build it yourself.
Afterwards you get binary in crates/wekan-cli/target/release/${platform}/wekan-cli to use.

# BUILD 


- Clone the repo
- Run one of the following steps:
    - `cargo build -r --features wekan-cli/store` and move it where you want it
    - `cargo install --features wekan-cli/store` and use it
    - `make use` and move it where you want it
    - `docker build -name wekan-cli -t wekan-cli:YOUR_TAG .` and `docker cp wekan-cli:/usr/local/cargo/bin/wekan-cli $PWD/wekan-cli` to install it via Docker

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

[DEVELOPMENT](./doc/DEVELOPMENT.md)

## TESTS

## Unittests

`cargo run test` for wekan-cli and wekan-core. WekanCommon doesn't need tests.


### E2E

[E2E](./doc/E2E.md)

### Coverage

[COVERAGE](./doc/COVERAGE.md)


## THANK YOU

Obviously, a lot of the usually research and thanks for the wonderful documentation from the RUST community, the rust book and especially the RUST compiler.  <3


# REFERENCES

- [WekanAPI v6.11](https://wekan.github.io/api/v6.11/#wekan-rest-api)
- [Wekan open-source Kanban](https://wekan.github.io/)
- [Github Wekan](https://github.com/wekan/wekan)


# LICENSE

Based on RUST and different crates. This project uses Apache and MIT license, please see the files [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT).
