NVIM ?= nvim
PLENARY_DIR ?= $(abspath ../../.deps/plenary.nvim)

.PHONY: lint test check

lint:
	stylua --check lua plugin tests
	selene --config selene.toml lua plugin
	selene --config selene-tests.toml tests

test:
	PLENARY_DIR="$(PLENARY_DIR)" $(NVIM) --headless --noplugin -u tests/minimal_init.lua \
		-c "lua require('plenary.test_harness').test_directory('tests/versionlens', { minimal_init = 'tests/minimal_init.lua' })" \
		-c "qa!"

check: lint test
