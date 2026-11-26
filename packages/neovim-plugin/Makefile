NVIM ?= nvim
SERVER ?= $(abspath ../../target/debug/versionlens-lsp)

.PHONY: lint test live check

lint:
	stylua --check lua plugin tests
	selene --config selene.toml lua plugin tests

test:
	cargo build -p versionlens-lsp --locked
	# Private data and state directories keep the user's settings, credentials, and lsp.log out.
	scratch=$$(mktemp -d) && XDG_DATA_HOME="$$scratch/data" XDG_STATE_HOME="$$scratch/state" \
		$(NVIM) --headless --clean -l tests/run.lua "$(SERVER)"

# Needs the network: the checks read the live registries and OSV.
live:
	cargo build -p versionlens-lsp --locked
	$(NVIM) --headless --clean -l tests/live.lua "$(SERVER)"

check: lint test
