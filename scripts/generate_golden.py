#!/usr/bin/env python3
"""Generate golden files for CNWS testing."""

import struct
import hashlib
import os

def generate_superblock():
    """Generate a golden SUPERBLOCK file."""
    magic = b"CNWSSB01"
    version = struct.pack("<I", 1)
    created_at = struct.pack("<Q", 1234567890)
    modified_at = struct.pack("<Q", 1234567890)
    segment_count = struct.pack("<I", 1)
    tile_count = struct.pack("<Q", 0)
    total_size = struct.pack("<Q", 4096)
    reserved = b"\x00" * 4052

    data = magic + version + created_at + modified_at + segment_count + tile_count + total_size + reserved
    assert len(data) == 4096, f"SUPERBLOCK size mismatch: {len(data)}"

    with open("fixtures/golden/superblock.cd", "wb") as f:
        f.write(data)

    print("Generated fixtures/golden/superblock.cd")

def generate_manifest():
    """Generate a golden MANIFEST file."""
    data = b'{"version":1,"tiles":0,"cells":0}'
    with open("fixtures/golden/manifest.cd", "wb") as f:
        f.write(data)
    print("Generated fixtures/golden/manifest.cd")

def generate_tile():
    """Generate a golden tile file."""
    data = b"test tile data"
    with open("fixtures/golden/tile.cd", "wb") as f:
        f.write(data)
    print("Generated fixtures/golden/tile.cd")

if __name__ == "__main__":
    os.makedirs("fixtures/golden", exist_ok=True)
    generate_superblock()
    generate_manifest()
    generate_tile()
    print("Golden files generated successfully!")
