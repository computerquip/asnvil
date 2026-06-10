//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./EmbeddedChoiceModule.rs"]
mod generated;

use generated::{Container, EmbeddedChoice, ExtendedSequence};
use num_bigint::BigInt;

fn main() {
    test_choice_integer();
    test_choice_string();
    test_choice_octet();
    test_container_with_choice_integer();
    test_container_with_choice_string();
    test_container_with_choice_octet();
    test_extended_sequence_base();
    test_extended_sequence_with_optional();
    test_choice_tag_uniqueness();
    println!("\nAll Embedded Choice integration tests passed!");
}

fn test_choice_integer() {
    let t = EmbeddedChoice::integer(BigInt::from(42));
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = EmbeddedChoice::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        EmbeddedChoice::integer(v) => assert_eq!(v, BigInt::from(42)),
        _ => panic!("Expected integer"),
    }
    println!("PASS: test_choice_integer");
}

fn test_choice_string() {
    let t = EmbeddedChoice::string("hello".to_string());
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = EmbeddedChoice::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        EmbeddedChoice::string(v) => assert_eq!(v, "hello"),
        _ => panic!("Expected string"),
    }
    println!("PASS: test_choice_string");
}

fn test_choice_octet() {
    let t = EmbeddedChoice::octet(vec![1, 2, 3]);
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = EmbeddedChoice::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        EmbeddedChoice::octet(v) => assert_eq!(v, vec![1, 2, 3]),
        _ => panic!("Expected octet"),
    }
    println!("PASS: test_choice_octet");
}

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
    println!("PASS: test_container_with_choice_integer");
}

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
    println!("PASS: test_container_with_choice_string");
}

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
    println!("PASS: test_container_with_choice_octet");
}

fn test_extended_sequence_base() {
    let t = ExtendedSequence { name: "base".to_string(), value: BigInt::from(1), extra: None };
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = ExtendedSequence::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "base");
    assert_eq!(decoded.value, BigInt::from(1));
    println!("PASS: test_extended_sequence_base");
}

fn test_extended_sequence_with_optional() {
    let t = ExtendedSequence { name: "extended".to_string(), value: BigInt::from(2), extra: Some("bonus".to_string()) };
    let encoded = t.encode_der().expect("Failed to encode");
    let decoded = ExtendedSequence::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "extended");
    assert_eq!(decoded.value, BigInt::from(2));
    assert_eq!(decoded.extra, Some("bonus".to_string()));
    println!("PASS: test_extended_sequence_with_optional");
}

fn test_choice_tag_uniqueness() {
    let t_int = EmbeddedChoice::integer(BigInt::from(1));
    let t_str = EmbeddedChoice::string("x".to_string());
    let encoded_int = t_int.encode_der().expect("Failed to encode");
    let encoded_str = t_str.encode_der().expect("Failed to encode");
    assert_ne!(encoded_int[0], encoded_str[0], "CHOICE alternatives should have distinct tags");
    println!("PASS: test_choice_tag_uniqueness");
}