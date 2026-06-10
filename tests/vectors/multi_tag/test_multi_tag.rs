//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./MultiTagModule.rs"]
mod generated;

use generated::{SingleImplicit, DoubleImplicit, SingleExplicit, DoubleExplicit, DeeplyTagged, MultipleTagged};
use num_bigint::BigInt;

#[test]
fn test_implicit_single() {
    let val = SingleImplicit { value: BigInt::from(42) };
    let encoded = val.encode_der().expect("Failed to encode");
    let decoded = SingleImplicit::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(42));
}

#[test]
fn test_implicit_double() {
    let val = DoubleImplicit { value: BigInt::from(99) };
    let encoded = val.encode_der().expect("Failed to encode");
    let decoded = DoubleImplicit::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(99));
}

#[test]
fn test_explicit_single() {
    let val = SingleExplicit { value: BigInt::from(100) };
    let encoded = val.encode_der().expect("Failed to encode");
    let decoded = SingleExplicit::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(100));
}

#[test]
fn test_explicit_double() {
    let val = DoubleExplicit { value: BigInt::from(200) };
    let encoded = val.encode_der().expect("Failed to encode");
    let decoded = DoubleExplicit::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(200));
}

#[test]
fn test_deeply_tagged_integer() {
    let val = DeeplyTagged { value: BigInt::from(300) };
    let encoded = val.encode_der().expect("Failed to encode");
    let decoded = DeeplyTagged::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(300));
}

#[test]
fn test_deeply_tagged_with_string() {
    let val = DeeplyTagged { value: BigInt::from(400) };
    let encoded = val.encode_der().expect("Failed to encode");
    let decoded = DeeplyTagged::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(400));
}

#[test]
fn test_tag_preserves_structure() {
    let val = MultipleTagged {
        field1: BigInt::from(1),
        field2: BigInt::from(2),
    };
    let encoded = val.encode_der().expect("Failed to encode");
    let decoded = MultipleTagged::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.field1, BigInt::from(1));
    assert_eq!(decoded.field2, BigInt::from(2));
}

#[test]
fn test_multiple_tagged_fields() {
    let val = MultipleTagged {
        field1: BigInt::from(10),
        field2: BigInt::from(20),
    };
    let encoded = val.encode_der().expect("Failed to encode");
    let decoded = MultipleTagged::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.field1, BigInt::from(10));
    assert_eq!(decoded.field2, BigInt::from(20));
}