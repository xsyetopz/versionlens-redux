NVIM ?= nvim
PLENARY_DIR ?= $(abspath ../../.deps/plenary.nvim)

.PHONY: lint test check

lint:
	stylua --check lua plugin tests
	selene lua plugin tests

test:
	PLENARY_DIR="$(PLENARY_DIR)" $(NVIM) --headless --noplugin -u tests/minimal_init.lua \
		-c "lua require('plenary.test_harness').test_directory('tests/versionlens', { minimal_init = 'tests/minimal_init.lua' })" \
		-c "qa!"

check: lint test
