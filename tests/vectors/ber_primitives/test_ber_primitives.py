"""Runtime BER primitive tests using YAML test vectors."""
import os
import sys
from pathlib import Path
import yaml
import pytest

from asnvil_runtime import Tag, TagClass, BerDecoder

# Load the test vectors from the co-located data.yaml file
VECTORS_PATH = Path(__file__).parent / "data.yaml"
with open(VECTORS_PATH) as f:
    VECTORS = yaml.safe_load(f)

# Filter to only include vectors that test tag decoding (have 'expected' with tag_class)
TAG_VECTORS = [v for v in VECTORS if v.get("expected", {}).get("tag_class") is not None]


@pytest.mark.parametrize("vector", TAG_VECTORS, ids=lambda v: v["name"])
def test_ber_tag_decoding(vector):
    """Test BER tag decoding against known vectors."""
    ber_bytes = bytes.fromhex(vector["ber_hex"])
    expected = vector["expected"]
    
    decoder = BerDecoder(ber_bytes)
    tag_class, tag_number, constructed = decoder.read_tag()
    
    # Map string tag class to enum
    expected_class_map = {
        "universal": TagClass.UNIVERSAL,
        "application": TagClass.APPLICATION,
        "context": TagClass.CONTEXT_SPECIFIC,
        "private": TagClass.PRIVATE,
    }
    
    assert tag_class == expected_class_map[expected["tag_class"].lower()], \
        f"Tag class mismatch for {vector['name']}"
    assert tag_number == expected["tag_number"], \
        f"Tag number mismatch for {vector['name']}"
    assert constructed == expected["constructed"], \
        f"Constructed flag mismatch for {vector['name']}"
