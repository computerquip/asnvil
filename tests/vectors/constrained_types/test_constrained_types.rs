//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./ConstrainedTypes.rs"]
mod generated;

use generated::UserRecord;
use num_bigint::BigInt;

fn main() {
    test_valid_user_record();
    test_valid_user_with_optional();
    test_valid_boundary_id_zero();
    test_valid_boundary_id_max();
    test_valid_name_min_length();
    test_valid_name_max_length();
    test_invalid_id_exceeds_max();
    test_invalid_age_negative();
    test_invalid_name_empty();
    println!("\nAll Constrained Types integration tests passed!");
}

fn test_valid_user_record() {
    let user = UserRecord {
        id: BigInt::from(42),
        name: "Alice".to_string(),
        age: BigInt::from(30),
        status: None,
        notes: None,
    };
    let data = user.encode_der().expect("Failed to encode");
    let decoded = UserRecord::decode_der(&data).expect("Failed to decode");
    assert_eq!(decoded.id, BigInt::from(42));
    assert_eq!(decoded.name, "Alice");
    assert_eq!(decoded.age, BigInt::from(30));
    println!("PASS: test_valid_user_record");
}

fn test_valid_user_with_optional() {
    let user = UserRecord {
        id: BigInt::from(100),
        name: "Bob".to_string(),
        age: BigInt::from(25),
        status: Some(BigInt::from(3)),
        notes: Some("Test notes".to_string()),
    };
    let data = user.encode_der().expect("Failed to encode");
    let decoded = UserRecord::decode_der(&data).expect("Failed to decode");
    assert_eq!(decoded.id, BigInt::from(100));
    assert_eq!(decoded.name, "Bob");
    assert_eq!(decoded.status, Some(BigInt::from(3)));
    assert_eq!(decoded.notes, Some("Test notes".to_string()));
    println!("PASS: test_valid_user_with_optional");
}

fn test_valid_boundary_id_zero() {
    let user = UserRecord {
        id: BigInt::from(0),
        name: "X".to_string(),
        age: BigInt::from(0),
        status: None,
        notes: None,
    };
    let data = user.encode_der().expect("Failed to encode");
    let decoded = UserRecord::decode_der(&data).expect("Failed to decode");
    assert_eq!(decoded.id, BigInt::from(0));
    println!("PASS: test_valid_boundary_id_zero");
}

fn test_valid_boundary_id_max() {
    let user = UserRecord {
        id: BigInt::from(1000),
        name: "Y".to_string(),
        age: BigInt::from(150),
        status: None,
        notes: None,
    };
    let data = user.encode_der().expect("Failed to encode");
    let decoded = UserRecord::decode_der(&data).expect("Failed to decode");
    assert_eq!(decoded.id, BigInt::from(1000));
    assert_eq!(decoded.age, BigInt::from(150));
    println!("PASS: test_valid_boundary_id_max");
}

fn test_valid_name_min_length() {
    let user = UserRecord {
        id: BigInt::from(1),
        name: "A".to_string(),
        age: BigInt::from(1),
        status: None,
        notes: None,
    };
    let data = user.encode_der().expect("Failed to encode");
    let decoded = UserRecord::decode_der(&data).expect("Failed to decode");
    assert_eq!(decoded.name, "A");
    println!("PASS: test_valid_name_min_length");
}

fn test_valid_name_max_length() {
    let user = UserRecord {
        id: BigInt::from(1),
        name: "A".repeat(50),
        age: BigInt::from(1),
        status: None,
        notes: None,
    };
    let data = user.encode_der().expect("Failed to encode");
    let decoded = UserRecord::decode_der(&data).expect("Failed to decode");
    assert_eq!(decoded.name.len(), 50);
    println!("PASS: test_valid_name_max_length");
}

fn test_invalid_id_exceeds_max() {
    let user = UserRecord {
        id: BigInt::from(1001),
        name: "Test".to_string(),
        age: BigInt::from(25),
        status: None,
        notes: None,
    };
    assert!(user.encode_der().is_err(), "Expected ConstraintViolationError for id > 1000");
    println!("PASS: test_invalid_id_exceeds_max");
}

fn test_invalid_age_negative() {
    let user = UserRecord {
        id: BigInt::from(1),
        name: "Test".to_string(),
        age: BigInt::from(-1),
        status: None,
        notes: None,
    };
    assert!(user.encode_der().is_err(), "Expected ConstraintViolationError for age < 0");
    println!("PASS: test_invalid_age_negative");
}

fn test_invalid_name_empty() {
    let user = UserRecord {
        id: BigInt::from(1),
        name: "".to_string(),
        age: BigInt::from(25),
        status: None,
        notes: None,
    };
    assert!(user.encode_der().is_err(), "Expected ConstraintViolationError for empty name");
    println!("PASS: test_invalid_name_empty");
}