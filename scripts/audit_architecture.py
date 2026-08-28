#!/usr/bin/env python3
"""Run the architecture-boundaries audit command."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

from architecture_audit.cli import main as _main

if __name__ == "__main__":
    raise SystemExit(_main())
