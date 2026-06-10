//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./EmbeddedChoiceModule.rs"]
mod generated;

use generated::{Container, EmbeddedChoice, ExtendedSequence};
use num_bigint::BigInt;

#[test]
fn test_choice_integer() {
    let t = EmbeddedChoice::integer(BigInt::from(42));
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = EmbeddedChoice::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        EmbeddedChoice::integer(v) => assert_eq!(v, BigInt::from(42)),
        _ => panic!("Expected integer"),
    }
}

#[test]
fn test_choice_string() {
    let t = EmbeddedChoice::string("hello".to_string());
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = EmbeddedChoice::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        EmbeddedChoice::string(v) => assert_eq!(v, "hello"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_choice_octet() {
    let t = EmbeddedChoice::octet(vec![1, 2, 3]);
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = EmbeddedChoice::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        EmbeddedChoice::octet(v) => assert_eq!(v, vec![1, 2, 3]),
        _ => panic!("Expected octet"),
    }
}

#[test]
fn test_container_with_choice_integer() {
    let choice = EmbeddedChoice::integer(BigInt::from(100));
    let c = Container { id: BigInt::from(1), value: choice };
    let encoded = c.encode_der().expect("Failed to encode");
    let decoded = Container::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.id, BigInt::from(1));
    match decoded.value {
        EmbeddedChoice::integer(v) => assert_eq!(v, BigInt::from(100)),
        _ => panic!("Expected integer"),
    }
}

#[test]
fn test_container_with_choice_string() {
    let choice = EmbeddedChoice::string("test data".to_string());
    let c = Container { id: BigInt::from(2), value: choice };
    let encoded = c.encode_der().expect("Failed to encode");
    let decoded = Container::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.id, BigInt::from(2));
    match decoded.value {
        EmbeddedChoice::string(v) => assert_eq!(v, "test data"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_container_with_choice_octet() {
    let choice = EmbeddedChoice::octet(vec![0xDE, 0xAD, 0xBE, 0xEF]);
    let c = Container { id: BigInt::from(3), value: choice };
    let encoded = c.encode_der().expect("Failed to encode");
    let decoded = Container::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.id, BigInt::from(3));
    match decoded.value {
        EmbeddedChoice::octet(v) => assert_eq!(v, vec![0xDE, 0xAD, 0xBE, 0xEF]),
        _ => panic!("Expected octet"),
    }
}

#[test]
fn test_extended_sequence_base() {
    let t = ExtendedSequence { name: "base".to_string(), value: BigInt::from(1), extra: None };
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = ExtendedSequence::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "base");
    assert_eq!(decoded.value, BigInt::from(1));
}

#[test]
fn test_extended_sequence_with_optional() {
    let t = ExtendedSequence { name: "extended".to_string(), value: BigInt::from(2), extra: Some("bonus".to_string()) };
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = ExtendedSequence::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "extended");
    assert_eq!(decoded.value, BigInt::from(2));
    assert_eq!(decoded.extra, Some("bonus".to_string()));
}

#[test]
fn test_choice_tag_uniqueness() {
    let t_int = EmbeddedChoice::integer(BigInt::from(1));
    let t_str = EmbeddedChoice::string("x".to_string());
    let encoded_int = t_int.encode_der().expect("Failed to encode");
    let encoded_str = t_str.encode_der().expect("Failed to encode");
    assert_ne!(encoded_int[0], encoded_str[0], "CHOICE alternatives should have distinct tags");
}