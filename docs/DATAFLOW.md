# Crates

Wekan-Cli -> Wekan-Core -> BE

Wekan-Common needs to reflect the API objects. Currently v6.11


# Wekan-Cli

## Dataflow

- wekan-cli: WekanParser and starting the main runner.
- runner: Each artifact has his own runner struct and argument struct. In the future, all of them should just use one instance and make use of the different traits in `command.rs`, which are currently in development.
- Credentials: Login and Logout.


## Util

### Rust

- command: Common traits run make fn and process flow consistent.
- subcommand: Common args struct for subcommands.


### Resolver

Query: Trait to look for an id of an artfifact, where only the name is supplied.


## Store

no-store: Option to disable store. This is part of the store feature.

Each context has his own cache.


# Wekan-Core

## Traits:

- Client: Each artifact should define his own api trait with a `fn new` and `fn set_base`.
- Authentication: Request a token and write it in the config.
- Operation and Artifact: Generic methods to apply get, post, delete and put method on one artifact or request all artifact of one kind.
- PreFlight request: Healthcheck the specified host.
- Satisfy: Apply the AType in a Vec<Artifact> depending on the called api.
- Persistence: Read and write config in a file.
- Store: Write succesffuly requests with an age and parent in a file.


# Wekan-Common

- Struct and traits for the api and cli.
- - Request body structs
- - Response structs
- Constraints for the artifacts. For example, each runner has constraints, which needs to be fulfilled before it can be run. ListRunner needs an board artifact.

Further it provides mock data.
