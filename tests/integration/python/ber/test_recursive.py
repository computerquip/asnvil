"""Integration tests for recursive types.

Tests recursive types (SET OF self, SEQUENCE OF self, recursive CHOICE).

Adapted from vlm/asn1c tests-c-compiler/43-recursion-OK.asn1 (MIT license).
https://github.com/vlm/asn1c
"""
from __future__ import annotations

import os
import sys

_integ_output = os.environ.get("INTEG_OUTPUT_DIR")
if _integ_output:
    sys.path.insert(0, _integ_output)

import pytest

try:
    from RecursionModule import RecursiveSeq, RecursiveSet, RecursiveChoice
    HAS_GENERATED = True
except ImportError:
    HAS_GENERATED = False


@pytest.mark.skipif(not HAS_GENERATED, reason="Generated types not available")
class TestRecursiveTypes:
    """Test recursive type encode/decode roundtrips."""

    def test_recursive_seq_single(self):
        """Test RecursiveSeq with no children."""
        t = RecursiveSeq(value=1, children=[])
        encoded = t.encode_der()
        decoded = RecursiveSeq.decode_der(encoded)
        assert decoded.value == 1
        assert decoded.children == []

    def test_recursive_seq_nested(self):
        """Test RecursiveSeq with nested children."""
        child = RecursiveSeq(value=2, children=[])
        parent = RecursiveSeq(value=1, children=[child])
        encoded = parent.encode_der()
        decoded = RecursiveSeq.decode_der(encoded)
        assert decoded.value == 1
        assert len(decoded.children) == 1
        assert decoded.children[0].value == 2

    def test_recursive_seq_deep(self):
        """Test RecursiveSeq with 3 levels of nesting."""
        leaf = RecursiveSeq(value=3, children=[])
        mid = RecursiveSeq(value=2, children=[leaf])
        root = RecursiveSeq(value=1, children=[mid])
        encoded = root.encode_der()
        decoded = RecursiveSeq.decode_der(encoded)
        assert decoded.value == 1
        assert decoded.children[0].value == 2
        assert decoded.children[0].children[0].value == 3

    def test_recursive_seq_multiple_children(self):
        """Test RecursiveSeq with multiple children at same level."""
        c1 = RecursiveSeq(value=10, children=[])
        c2 = RecursiveSeq(value=20, children=[])
        c3 = RecursiveSeq(value=30, children=[])
        root = RecursiveSeq(value=0, children=[c1, c2, c3])
        encoded = root.encode_der()
        decoded = RecursiveSeq.decode_der(encoded)
        assert len(decoded.children) == 3
        assert [c.value for c in decoded.children] == [10, 20, 30]

    def test_recursive_set_single(self):
        """Test RecursiveSet with no children."""
        t = RecursiveSet(value=42, children=[])
        encoded = t.encode_der()
        decoded = RecursiveSet.decode_der(encoded)
        assert decoded.value == 42

    def test_recursive_set_nested(self):
        """Test RecursiveSet with nested children."""
        child = RecursiveSet(value=2, children=[])
        parent = RecursiveSet(value=1, children=[child])
        encoded = parent.encode_der()
        decoded = RecursiveSet.decode_der(encoded)
        assert decoded.value == 1
        assert len(decoded.children) == 1
        assert decoded.children[0].value == 2

    def test_recursive_choice_self(self):
        """Test RecursiveChoice selecting self-referencing alternative."""
        inner = RecursiveChoice(leaf=99)
        outer = RecursiveChoice(nested=inner)
        encoded = outer.encode_der()
        decoded = RecursiveChoice.decode_der(encoded)
        assert decoded.nested is not None
        assert decoded.nested.leaf == 99

    def test_recursive_choice_leaf(self):
        """Test RecursiveChoice selecting leaf alternative."""
        t = RecursiveChoice(leaf=42)
        encoded = t.encode_der()
        decoded = RecursiveChoice.decode_der(encoded)
        assert decoded.leaf == 42

    def test_recursive_choice_deep_nesting(self):
        """Test RecursiveChoice with 5 levels of nesting."""
        current = RecursiveChoice(leaf=5)
        for i in range(4, 0, -1):
            current = RecursiveChoice(nested=current)
        encoded = current.encode_der()
        decoded = RecursiveChoice.decode_der(encoded)
        for _ in range(4):
            assert decoded.nested is not None
            decoded = decoded.nested
        assert decoded.leaf == 5
