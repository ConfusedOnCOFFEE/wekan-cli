SHELL := /usr/bin/env bash

CARGO ?= cargo
FEATURE_STORE ?= --features wekan-cli/store
CARGO_RELEASE_FLAG ?= -r
FEATURE_INTEGRATION ?= --features wekan-cli/integration
CLIPPY_FLAGS=-Dwarnings
ci: test clippy fmt build

test:
	@echo "Testing..."
	$(CARGO) test
	$(CARGO) test $(FEATURE_STORE)
clippy:
	@echo "Run cargo clippy..."
	$(CARGO) clippy
	$(CARGO) clippy $(FEATURE_STORE) -- $(CLIPPY_FLAGS)

fmt:
	@echo "Run cargo fmt..."
	$(CARGO) fmt
build:
	$(CARGO) build
	$(CARGO) build $(FEATURE_STORE)
	$(CARGO) build $(FEATURE_INTEGRATION)

use:
	$(CARGO) build $(CARGO_RELEASE_FLAG) $(FEATURE_STORE)
