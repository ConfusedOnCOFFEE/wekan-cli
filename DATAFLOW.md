
## Crates
Wekan-Cli -> Wekan-Core -> BE

Wekan-Common needs to reflect the API objects. Currently v6.11


## Dataflow

main -> Logging, MainContect(cli.rs)
cli -> WekanParser, and subcommand.
login -> Load token, try to login via ENV_VARIABLES.(Should be removed.)
context -> Tries to make use of the supplied command, args and hands the work over to the sub-contexts.

## Util

### Rust

result -> WekanResult
error -> Errors and from Traits
artifact -> Common traits/struct and args/subcommand. Query as well.


### Wekan

Query:
Trait to look for an id of an artfifact, where only the name is supplied.


## Cache

cache: Args to disable/enable store. And store settings.

Motivation: It should be possible to have a complete second set supplied.
For example:
Try to use the store with host:port first, if that is not working, go use the settings via ENV_VARIABLE.


item: TODO
checklist: TODO
user: TODO garbage code
