#!/usr/bin/env python3
"""Public inline-test detection API."""

from .inline.finder import inline_test_findings
from .inline.source import is_test_source

__all__ = ["inline_test_findings", "is_test_source"]
