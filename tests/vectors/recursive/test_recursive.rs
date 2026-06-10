//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./RecursionModule.rs"]
mod generated;

use generated::{RecursiveSeq, RecursiveSet, RecursiveChoice};
use num_bigint::BigInt;

#[test]
fn test_recursive_seq_single() {
    let node = RecursiveSeq {
        value: BigInt::from(42),
        children: vec![],
    };
    let encoded = node.encode_der().expect("Failed to encode");
    let decoded = RecursiveSeq::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(42));
    assert!(decoded.children.is_empty());
}

#[test]
fn test_recursive_seq_nested() {
    let child = RecursiveSeq {
        value: BigInt::from(10),
        children: vec![],
    };
    let parent = RecursiveSeq {
        value: BigInt::from(20),
        children: vec![child],
    };
    let encoded = parent.encode_der().expect("Failed to encode");
    let decoded = RecursiveSeq::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(20));
    assert_eq!(decoded.children.len(), 1);
    assert_eq!(decoded.children[0].value, BigInt::from(10));
}

#[test]
fn test_recursive_seq_deep() {
    let mut current = RecursiveSeq {
        value: BigInt::from(0),
        children: vec![],
    };
    for i in 1..5 {
        current = RecursiveSeq {
            value: BigInt::from(i),
            children: vec![current],
        };
    }
    let encoded = current.encode_der().expect("Failed to encode");
    let decoded = RecursiveSeq::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(4));
    assert_eq!(decoded.children[0].value, BigInt::from(3));
}

#[test]
fn test_recursive_set_single() {
    let node = RecursiveSet {
        value: BigInt::from(99),
        children: vec![],
    };
    let encoded = node.encode_der().expect("Failed to encode");
    let decoded = RecursiveSet::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(99));
    assert!(decoded.children.is_empty());
}

#[test]
fn test_recursive_set_nested() {
    let child = RecursiveSet {
        value: BigInt::from(5),
        children: vec![],
    };
    let parent = RecursiveSet {
        value: BigInt::from(15),
        children: vec![child],
    };
    let encoded = parent.encode_der().expect("Failed to encode");
    let decoded = RecursiveSet::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.value, BigInt::from(15));
    assert_eq!(decoded.children.len(), 1);
    assert_eq!(decoded.children[0].value, BigInt::from(5));
}

#[test]
fn test_recursive_choice_self() {
    let inner = RecursiveChoice::Leaf(BigInt::from(77));
    let outer = RecursiveChoice::Node {
        value: BigInt::from(88),
        child: Box::new(inner),
    };
    let encoded = outer.encode_der().expect("Failed to encode");
    let decoded = RecursiveChoice::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        RecursiveChoice::Node { value, child } => {
            assert_eq!(value, BigInt::from(88));
            match *child {
                RecursiveChoice::Leaf(v) => assert_eq!(v, BigInt::from(77)),
                _ => panic!("Expected Leaf"),
            }
        }
        _ => panic!("Expected Node"),
    }
}

#[test]
fn test_recursive_choice_leaf() {
    let leaf = RecursiveChoice::Leaf(BigInt::from(123));
    let encoded = leaf.encode_der().expect("Failed to encode");
    let decoded = RecursiveChoice::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        RecursiveChoice::Leaf(v) => assert_eq!(v, BigInt::from(123)),
        _ => panic!("Expected Leaf"),
    }
}