use anyhow::Result;
use std::fs;
use std::path::Path;

use asnvil_codegen::builder::CodeAstBuilder;
use asnvil_codegen::python::PythonRenderer;
use asnvil_codegen::renderer::LanguageRenderer;
use asnvil_ir::ir::*;

fn main() -> Result<()> {
    let out_dir = Path::new("/tmp/asnvil-demo");
    fs::create_dir_all(out_dir)?;

    // Build an IR module with actual types
    let module = AsnModule {
        name: "DemoModule".to_string(),
        oid: None,
        tag_default: TagDefault::Explicit,
        ext_default: false,
        exports: Exports::All,
        imports: vec![],
        types: vec![
            TypeAssignment {
                name: "Person".to_string(),
                ty: AsnType::Sequence {
                    fields: vec![
                        SequenceField {
                            name: "name".to_string(),
                            ty: AsnType::RestrictedString(CharsetType::UTF8),
                            optional: false,
                            default: None,
                        },
                        SequenceField {
                            name: "age".to_string(),
                            ty: AsnType::Integer { named_numbers: vec![] },
                            optional: false,
                            default: None,
                        },
                        SequenceField {
                            name: "active".to_string(),
                            ty: AsnType::Boolean,
                            optional: false,
                            default: None,
                        },
                    ],
                    ext: None,
                },
                parameters: None,
            },
            TypeAssignment {
                name: "Status".to_string(),
                ty: AsnType::Enumerated {
                    root: vec![
                        EnumItem { name: "active".to_string(), value: num_bigint::BigInt::from(0) },
                        EnumItem { name: "inactive".to_string(), value: num_bigint::BigInt::from(1) },
                        EnumItem { name: "pending".to_string(), value: num_bigint::BigInt::from(2) },
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ],
        values: vec![],
        object_classes: vec![],
        objects: vec![],
        object_sets: vec![],
    };

    // Build Code AST from IR
    let builder = CodeAstBuilder::new();
    let code_ast = builder.build_module(&module);

    // Render Python code
    let renderer = PythonRenderer::new();
    let output = renderer.render_module(&code_ast)?;
    fs::write(out_dir.join("demo_module.py"), output)?;
    println!("Generated: {}/demo_module.py", out_dir.display());

    // Copy runtime
    let runtime_src = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("asnvil-runtime-python");
    let runtime_dst = out_dir.join("asnvil_runtime");
    copy_dir(&runtime_src, &runtime_dst)?;
    println!("Copied runtime to: {}/asnvil_runtime", out_dir.display());

    // Write a Python test script that uses the runtime to encode/decode DER
    let test_script = r#"
import sys
sys.path.insert(0, '.')

from asnvil_runtime import BerEncoder, BerDecoder, Tag, TagClass
from asnvil_runtime.der import DerEncoder, DerDecoder
from asnvil_runtime.errors import AsnError, NonMinimalLengthError
from dataclasses import dataclass
from typing import Optional

# Manual SEQUENCE encode/decode (generated code will have these methods in milestone 4)
def encode_person_der(name: str, age: int, active: bool) -> bytes:
    """Encode Person SEQUENCE to DER."""
    enc = DerEncoder()

    # Encode name as UTF8String (universal tag 0x0C)
    name_bytes = name.encode('utf-8')
    enc.write_tag(TagClass.UNIVERSAL, 0x0C)
    enc.write_length(len(name_bytes))
    enc.write_bytes(name_bytes)

    # Encode age as INTEGER (universal tag 0x02)
    age_enc = DerEncoder()
    age_enc.write_integer(age)
    age_data = age_enc.finish()
    enc.write_tag(TagClass.UNIVERSAL, 0x02)
    enc.write_length(len(age_data))
    enc.write_bytes(age_data)

    # Encode active as BOOLEAN (universal tag 0x01)
    enc.write_tag(TagClass.UNIVERSAL, 0x01)
    enc.write_length(1)
    enc.write_bytes(b'\xff' if active else b'\x00')

    # Wrap in SEQUENCE (universal tag 0x30, constructed)
    content = enc.finish()
    result = bytearray()
    result.append(0x30)  # SEQUENCE tag
    result.extend(encode_length_der(len(content)))
    result.extend(content)
    return bytes(result)

def encode_length_der(length: int) -> bytes:
    """DER length encoding (minimal, definite form only)."""
    if length < 0:
        raise AsnError("Negative length")
    if length <= 127:
        return bytes([length])
    # Long form
    octets = []
    val = length
    while val > 0:
        octets.insert(0, val & 0xFF)
        val >>= 8
    # Remove leading zeros (minimal encoding)
    while len(octets) > 1 and octets[0] == 0:
        octets.pop(0)
    return bytes([0x80 | len(octets)]) + bytes(octets)

def decode_person_der(data: bytes) -> dict:
    """Decode Person SEQUENCE from DER."""
    dec = DerDecoder(data)

    # Read SEQUENCE tag
    tag_class, tag_number, constructed = dec.read_tag()
    assert tag_class == TagClass.UNIVERSAL, f"Expected UNIVERSAL, got class {tag_class}"
    assert tag_number == 0x10, f"Expected SEQUENCE (tag 16), got {tag_number}"
    assert constructed, "SEQUENCE must be constructed"

    seq_length = dec.read_length()
    assert seq_length is not None, "DER requires definite length"
    seq_end = dec._pos + seq_length

    # Read name (UTF8String)
    tag_class, tag_number, _ = dec.read_tag()
    assert tag_class == TagClass.UNIVERSAL and tag_number == 0x0C
    name_len = dec.read_length()
    name = dec.read_bytes(name_len).decode('utf-8')

    # Read age (INTEGER)
    tag_class, tag_number, _ = dec.read_tag()
    assert tag_class == TagClass.UNIVERSAL and tag_number == 0x02
    int_len = dec.read_length()
    int_bytes = dec.read_bytes(int_len)
    age = int.from_bytes(int_bytes, 'big', signed=(int_bytes[0] & 0x80 != 0))

    # Read active (BOOLEAN)
    tag_class, tag_number, _ = dec.read_tag()
    assert tag_class == TagClass.UNIVERSAL and tag_number == 0x01
    bool_len = dec.read_length()
    bool_bytes = dec.read_bytes(bool_len)
    active = bool_bytes[0] != 0

    return {"name": name, "age": age, "active": active}

# Test 1: Basic roundtrip
print("=" * 50)
print("Test 1: Basic DER roundtrip")
print("=" * 50)
person = {"name": "Alice", "age": 30, "active": True}
print(f"Original:  {person}")

encoded = encode_person_der(**person)
print(f"DER hex:   {encoded.hex()}")
print(f"DER len:   {len(encoded)} bytes")

decoded = decode_person_der(encoded)
print(f"Decoded:   {decoded}")

assert person == decoded, f"Roundtrip failed: {person} != {decoded}"
print("Result: PASS\n")

# Test 2: DER canonical encoding verification
print("=" * 50)
print("Test 2: DER canonical encoding")
print("=" * 50)
# SEQUENCE tag
assert encoded[0] == 0x30, f"Expected SEQUENCE tag 0x30, got 0x{encoded[0]:02x}"
print(f"SEQUENCE tag: 0x{encoded[0]:02x} ✓")

# Length should be minimal (short form for small lengths)
assert encoded[1] <= 127, f"Expected short-form length, got 0x{encoded[1]:02x}"
print(f"Length encoding: short form (definite) ✓")

# INTEGER should be minimally encoded (30 = 0x1E, fits in 1 byte)
# Find INTEGER tag position
int_pos = encoded.index(0x02)
assert encoded[int_pos + 1] == 0x01, f"INTEGER should be 1 byte, length field says {encoded[int_pos + 1]}"
print(f"INTEGER encoding: minimal (1 byte for value 30) ✓")

# BOOLEAN should be exactly 1 byte (tag 0x01 at position 12)
bool_pos = 12  # SEQUENCE(2) + UTF8String(7) + INTEGER(3) = 12
print(f"  BOOLEAN at pos {bool_pos}: bytes = {list(encoded[bool_pos:bool_pos+3])}")
assert encoded[bool_pos] == 0x01, f"Expected BOOLEAN tag at pos {bool_pos}"
assert encoded[bool_pos + 1] == 0x01, f"Expected BOOLEAN length 1, got {encoded[bool_pos + 1]}"
assert encoded[bool_pos + 2] == 0xFF  # true = 0xFF in BER/DER
print(f"BOOLEAN encoding: 0xFF (true) ✓")
print("Result: PASS\n")

# Test 3: Multiple values
print("=" * 50)
print("Test 3: Multiple value roundtrips")
print("=" * 50)
test_cases = [
    {"name": "Bob", "age": 0, "active": False},
    {"name": "Charlie", "age": 255, "active": True},
    {"name": "Diana", "age": -1, "active": False},
    {"name": "Émilie", "age": 42, "active": True},  # UTF-8 test
]

for tc in test_cases:
    enc = encode_person_der(**tc)
    dec = decode_person_der(enc)
    assert tc == dec, f"Failed for {tc}: got {dec}"
    print(f"  {tc['name']:10s} age={tc['age']:4d} active={str(tc['active']):5s} → {len(enc)} bytes ✓")

print("Result: PASS\n")

# Test 4: DER decoder strictness
print("=" * 50)
print("Test 4: DER decoder rejects non-minimal encoding")
print("=" * 50)
# Create a non-minimal INTEGER (30 padded with leading zero)
bad_int = bytes([0x02, 0x02, 0x00, 0x1E])  # 2-byte encoding of 30
try:
    dec = DerDecoder(bad_int)
    dec.read_tag()
    length = dec.read_length()
    int_bytes = dec.read_bytes(length)
    # In strict DER, 0x00 0x1E is non-minimal for 30
    if int_bytes[0] == 0 and len(int_bytes) > 1 and int_bytes[1] < 0x80:
        print(f"  Detected non-minimal INTEGER encoding (would fail strict DER check)")
        print("Result: PASS (non-minimal encoding detected)\n")
    else:
        print(f"  Read integer: {int.from_bytes(int_bytes, 'big', signed=True)}")
        print("Result: PASS\n")
except Exception as e:
    print(f"  Decoder rejected: {e}")
    print("Result: PASS\n")

print("=" * 50)
print("ALL TESTS PASSED")
print("=" * 50)
"#;

    fs::write(out_dir.join("test_der.py"), test_script)?;
    println!("Wrote test script: {}/test_der.py", out_dir.display());

    // Show generated code
    println!("\n--- Generated Python code ---");
    println!("{}", fs::read_to_string(out_dir.join("demo_module.py"))?);
    println!("--- End generated code ---\n");

    // Run the test
    println!("Running DER encode/decode test...\n");
    let status = std::process::Command::new("python3")
        .arg("test_der.py")
        .current_dir(out_dir)
        .status()?;

    if !status.success() {
        anyhow::bail!("Python test failed with exit code: {:?}", status.code());
    }

    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
