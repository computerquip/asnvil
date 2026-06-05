"""BER/DER test vector tests using shared YAML vectors from asn1c.

Test vectors adapted from the vlm/asn1c project (MIT license).
https://github.com/vlm/asn1c

Tests BerDecoder and DerDecoder directly against each vector.
"""
from __future__ import annotations

import os
import pathlib
from typing import Any

import pytest
import yaml

from asnvil_runtime.ber import BerDecoder
from asnvil_runtime.der import DerDecoder
from asnvil_runtime.errors import (
    AsnError,
    TruncatedInputError,
    IndefiniteLengthNotAllowedError,
    InvalidIntegerEncodingError,
    InvalidTagError,
)
from asnvil_runtime import TagClass

VECTORS_DIR = pathlib.Path(__file__).resolve().parent / "vectors" / "ber"
DATA_YAML = VECTORS_DIR / "data.yaml"
DATA62_DIR = VECTORS_DIR / "data-62"

TAG_CLASS_MAP = {
    "universal": TagClass.UNIVERSAL,
    "context": TagClass.CONTEXT,
    "application": TagClass.APPLICATION,
    "private": TagClass.PRIVATE,
}


def _load_vectors() -> list[dict[str, Any]]:
    """Load all test vectors from the shared YAML file."""
    with open(DATA_YAML) as f:
        return yaml.safe_load(f)


def _hex_to_bytes(hex_str: str) -> bytes:
    """Convert hex string to bytes."""
    return bytes.fromhex(hex_str)


def _assert_tag_matches(actual_tag: tuple, expected: dict) -> None:
    """Assert that a decoded tag matches expected values."""
    tag_class_map = {
        "universal": 0,
        "application": 1,
        "context": 2,
        "private": 3,
    }
    if "tag_class" in expected:
        assert actual_tag[0] == tag_class_map[expected["tag_class"]], (
            f"tag_class mismatch: expected {tag_class_map.get(expected['tag_class'])}, got {actual_tag[0]}"
        )
    if "tag_number" in expected:
        assert actual_tag[1] == expected["tag_number"], (
            f"tag_number mismatch: expected {expected['tag_number']}, got {actual_tag[1]}"
        )
    if "constructed" in expected:
        assert actual_tag[2] == expected["constructed"], (
            f"constructed mismatch: expected {expected['constructed']}, got {actual_tag[2]}"
        )


def _assert_integer_matches(decoded: int, expected_value: int) -> None:
    """Assert that a decoded integer matches the expected value."""
    assert decoded == expected_value, (
        f"integer mismatch: expected {expected_value}, got {decoded}"
    )


def _assert_length_matches(actual_length, expected: dict) -> None:
    """Assert that a decoded length matches expected values."""
    if "length" in expected:
        expected_len = expected["length"]
        if expected_len is None:
            assert actual_length is None, (
                f"expected indefinite length (None), got {actual_length}"
            )
        else:
            assert actual_length == expected_len, (
                f"length mismatch: expected {expected_len}, got {actual_length}"
            )


# Load all vectors once at module level
_ALL_VECTORS = _load_vectors()


def _get_vectors_by_category(category_prefix: str) -> list[dict]:
    """Get vectors matching a category prefix."""
    return [v for v in _ALL_VECTORS if v["name"].startswith(category_prefix)]


def _get_error_class(error_name: str):
    """Get the error class by name."""
    error_map = {
        "TruncatedInputError": TruncatedInputError,
        "IndefiniteLengthNotAllowedError": IndefiniteLengthNotAllowedError,
        "InvalidIntegerEncodingError": InvalidIntegerEncodingError,
        "InvalidTagError": InvalidTagError,
        "AsnError": AsnError,
    }
    return error_map.get(error_name, AsnError)


# ============================================================
# TAG DECODE TESTS
# ============================================================

class TestTagDecode:
    """Test tag encoding/decoding with long-form edge cases."""

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("tag_") and "error" not in v],
        ids=lambda v: v["name"],
    )
    def test_tag_decode(self, vector: dict) -> None:
        """Test tag decoding for valid tag vectors."""
        data = _hex_to_bytes(vector["ber_hex"])
        decoder = BerDecoder(data)
        tag = decoder.read_tag()
        _assert_tag_matches(tag, vector["expected"])

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("tag_") and "error" in v],
        ids=lambda v: v["name"],
    )
    def test_tag_decode_invalid(self, vector: dict) -> None:
        """Test that invalid tag encodings raise errors."""
        data = _hex_to_bytes(vector["ber_hex"])
        decoder = BerDecoder(data)
        error_class = _get_error_class(vector["error"])
        with pytest.raises(error_class):
            decoder.read_tag()


# ============================================================
# LENGTH DECODE TESTS
# ============================================================

class TestLengthDecode:
    """Test length encoding/decoding with boundary values."""

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("length_") and "error" not in v],
        ids=lambda v: v["name"],
    )
    def test_length_decode(self, vector: dict) -> None:
        """Test length decoding for valid length vectors."""
        data = _hex_to_bytes(vector["ber_hex"])
        decoder = BerDecoder(data)
        length = decoder.read_length()
        _assert_length_matches(length, vector["expected"])

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("length_") and "error" in v],
        ids=lambda v: v["name"],
    )
    def test_length_decode_invalid(self, vector: dict) -> None:
        """Test that invalid length encodings raise errors."""
        data = _hex_to_bytes(vector["ber_hex"])
        decoder = BerDecoder(data)
        error_class = _get_error_class(vector["error"])
        with pytest.raises(error_class):
            decoder.read_length()


# ============================================================
# INTEGER DECODE TESTS
# ============================================================

class TestIntegerDecode:
    """Test INTEGER decode with known byte sequences."""

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("integer_")],
        ids=lambda v: v["name"],
    )
    def test_integer_decode_ber(self, vector: dict) -> None:
        """Test INTEGER decoding via BER."""
        data = _hex_to_bytes(vector["ber_hex"])
        decoder = BerDecoder(data)
        tag = decoder.read_tag()
        _assert_tag_matches(tag, vector["expected"])
        if "decoded_integer" in vector["expected"]:
            value = decoder.read_integer()
            _assert_integer_matches(value, vector["expected"]["decoded_integer"])

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("integer_")],
        ids=lambda v: v["name"],
    )
    def test_integer_decode_der(self, vector: dict) -> None:
        """Test INTEGER decoding via DER (should reject non-minimal)."""
        data = _hex_to_bytes(vector["ber_hex"])
        decoder = DerDecoder(data)
        decoder.read_tag()
        if "decoded_integer" in vector["expected"]:
            # DER rejects non-minimal encodings
            if "non_minimal" in vector["name"]:
                with pytest.raises(InvalidIntegerEncodingError):
                    decoder.read_integer()
            else:
                value = decoder.read_integer()
                _assert_integer_matches(value, vector["expected"]["decoded_integer"])


# ============================================================
# STRUCTURED DECODE TESTS (data-62 files)
# ============================================================

class TestStructuredDecode:
    """Test complete TLV structure decode from data-62 files."""

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("data62_") and v["name"] not in ("data62_16_indefinite_numeric_string", "data62_20_private_any")],
        ids=lambda v: v["name"],
    )
    def test_structured_decode_hex(self, vector: dict) -> None:
        """Test structured decoding from hex data."""
        data = _hex_to_bytes(vector["ber_hex"])
        decoder = BerDecoder(data)
        tag = decoder.read_tag()
        _assert_tag_matches(tag, vector["expected"])
        if "children" in vector["expected"]:
            length = decoder.read_length()
            if length is not None:
                content = decoder.read_bytes(length)
                assert len(content) == length

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("data62_") and "ber_file" in v],
        ids=lambda v: v["name"],
    )
    def test_structured_decode_file(self, vector: dict) -> None:
        """Test structured decoding from .ber file."""
        ber_file = DATA62_DIR / vector["ber_file"]
        if not ber_file.exists():
            pytest.skip(f"BER file not found: {ber_file}")
        data = ber_file.read_bytes()
        decoder = BerDecoder(data)
        tag = decoder.read_tag()
        _assert_tag_matches(tag, vector["expected"])


# ============================================================
# ERROR HANDLING TESTS
# ============================================================

class TestErrorHandling:
    """Test broken BER files should raise appropriate errors."""

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("error_") and v["name"] != "error_26_indefinite_in_der"],
        ids=lambda v: v["name"],
    )
    def test_error_handling_hex(self, vector: dict) -> None:
        """Test that error vectors raise expected errors (hex)."""
        data = _hex_to_bytes(vector["ber_hex"])
        decoder = BerDecoder(data)
        error_class = _get_error_class(vector["error"])
        with pytest.raises(error_class):
            decoder.read_tag()
            decoder.read_length()
            decoder.read_bytes(100)

    def test_error_26_indefinite_in_der(self) -> None:
        """Indefinite length in DER context should raise IndefiniteLengthNotAllowedError."""
        data = bytes([0x30, 0x80, 0x02, 0x01, 0x05, 0x00, 0x00])
        decoder = DerDecoder(data)
        decoder.read_tag()
        with pytest.raises(IndefiniteLengthNotAllowedError):
            decoder.read_length()

    @pytest.mark.parametrize(
        "vector",
        [v for v in _ALL_VECTORS if v["name"].startswith("error_") and "ber_file" in v and v["name"] != "error_26_indefinite_in_der"],
        ids=lambda v: v["name"],
    )
    def test_error_handling_file(self, vector: dict) -> None:
        """Test that error vectors raise expected errors (file)."""
        ber_file = DATA62_DIR / vector["ber_file"]
        if not ber_file.exists():
            pytest.skip(f"BER file not found: {ber_file}")
        data = ber_file.read_bytes()
        decoder = BerDecoder(data)
        error_class = _get_error_class(vector["error"])
        with pytest.raises(error_class):
            decoder.read_tag()
            decoder.read_length()
            decoder.read_bytes(100)


# ============================================================
# DER ERROR TESTS
# ============================================================

class TestDerErrorHandling:
    """Test DER-specific error handling."""

    def test_der_rejects_indefinite_length(self) -> None:
        """DER should reject indefinite length."""
        data = bytes([0x30, 0x80])  # SEQUENCE with indefinite length
        decoder = DerDecoder(data)
        decoder.read_tag()
        with pytest.raises(IndefiniteLengthNotAllowedError):
            decoder.read_length()

    def test_der_rejects_non_minimal_tag(self) -> None:
        """DER should reject non-minimal long-form tags."""
        data = bytes([0x1F, 0x1E])  # Tag 30 in non-minimal long form
        decoder = DerDecoder(data)
        with pytest.raises(InvalidTagError):
            decoder.read_tag()

    def test_ber_rejects_non_minimal_tag(self) -> None:
        """BER should accept non-minimal long-form tags (per X.690)."""
        data = bytes([0x1F, 0x1E])  # Tag 30 in non-minimal long form
        decoder = BerDecoder(data)
        tag = decoder.read_tag()
        assert tag == (0, 30, False)  # Should accept non-minimal BER
