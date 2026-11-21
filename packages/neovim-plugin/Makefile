NVIM ?= nvim
SERVER ?= $(abspath ../../target/debug/versionlens-lsp)

.PHONY: lint test check

lint:
	stylua --check lua plugin tests
	selene --config selene.toml lua plugin tests

test:
	cargo build -p versionlens-lsp --locked
	$(NVIM) --headless --clean -l tests/run.lua "$(SERVER)"

check: lint test
