# ARCHITECTURE

This repo contains three crates.

- wekan-cli uses clap to build a CLI with the help of derive API
- wekan-core uses mostly reqwest crate to do the heavy lifting of http requests.
- wekan-common keeps all the structs comming from the Wekan API and structs to build bodies for requests.

Motivation:

Not so sure, if that is a good way right now, but it could help if I upgrade to a new API version. Also wekan-core can be reused and not use it with the CLI.


# DEVELOPMENT

# EDITOR

Emacs:

Use `./manager.sh d` if you use emacs. If will crate .emacs.desktop files inside crates/wekan-cli to have a extra workspace for this project.

