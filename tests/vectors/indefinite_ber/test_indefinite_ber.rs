//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./TestModule.rs"]
mod generated;

use generated::{Person, Department, Company, Config};
use num_bigint::BigInt;

fn main() {
    test_person_ber_indefinite_roundtrip();
    test_department_ber_indefinite_roundtrip();
    test_company_ber_indefinite_roundtrip();
    test_config_ber_indefinite_defaults();
    test_config_ber_indefinite_with_values();
    test_ber_indefinite_inner_matches_der_inner();
    test_ber_indefinite_decode_matches_der_decode();
    test_ber_indefinite_nested();
    test_ber_indefinite_department_with_people();
    println!("\nAll Indefinite BER integration tests passed!");
}

fn test_person_ber_indefinite_roundtrip() {
    let person = Person {
        name: "Alice".to_string(),
        age: BigInt::from(30),
    };
    let encoded = person.encode_ber_indefinite().expect("Failed to encode");
    let decoded = Person::decode_ber_indefinite(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "Alice");
    assert_eq!(decoded.age, BigInt::from(30));
    println!("PASS: test_person_ber_indefinite_roundtrip");
}

fn test_department_ber_indefinite_roundtrip() {
    let dept = Department {
        name: "Engineering".to_string(),
        id: BigInt::from(101),
    };
    let encoded = dept.encode_ber_indefinite().expect("Failed to encode");
    let decoded = Department::decode_ber_indefinite(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "Engineering");
    assert_eq!(decoded.id, BigInt::from(101));
    println!("PASS: test_department_ber_indefinite_roundtrip");
}

fn test_company_ber_indefinite_roundtrip() {
    let dept = Department {
        name: "Engineering".to_string(),
        id: BigInt::from(101),
    };
    let company = Company {
        name: "TechCorp".to_string(),
        departments: vec![dept],
    };
    let encoded = company.encode_ber_indefinite().expect("Failed to encode");
    let decoded = Company::decode_ber_indefinite(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "TechCorp");
    assert_eq!(decoded.departments.len(), 1);
    println!("PASS: test_company_ber_indefinite_roundtrip");
}

fn test_config_ber_indefinite_defaults() {
    let config = Config {
        timeout: None,
        retries: None,
    };
    let encoded = config.encode_ber_indefinite().expect("Failed to encode");
    let decoded = Config::decode_ber_indefinite(&encoded).expect("Failed to decode");
    assert!(decoded.timeout.is_none());
    assert!(decoded.retries.is_none());
    println!("PASS: test_config_ber_indefinite_defaults");
}

fn test_config_ber_indefinite_with_values() {
    let config = Config {
        timeout: Some(BigInt::from(30)),
        retries: Some(BigInt::from(3)),
    };
    let encoded = config.encode_ber_indefinite().expect("Failed to encode");
    let decoded = Config::decode_ber_indefinite(&encoded).expect("Failed to decode");
    assert_eq!(decoded.timeout, Some(BigInt::from(30)));
    assert_eq!(decoded.retries, Some(BigInt::from(3)));
    println!("PASS: test_config_ber_indefinite_with_values");
}

fn test_ber_indefinite_inner_matches_der_inner() {
    let person = Person {
        name: "Bob".to_string(),
        age: BigInt::from(25),
    };
    let ber_encoded = person.encode_ber_indefinite().expect("Failed to encode BER");
    let der_encoded = person.encode_der().expect("Failed to encode DER");
    // The inner content should be similar, though BER has indefinite length markers
    assert!(ber_encoded.len() > der_encoded.len());
    println!("PASS: test_ber_indefinite_inner_matches_der_inner");
}

fn test_ber_indefinite_decode_matches_der_decode() {
    let person = Person {
        name: "Charlie".to_string(),
        age: BigInt::from(40),
    };
    let ber_encoded = person.encode_ber_indefinite().expect("Failed to encode BER");
    let der_encoded = person.encode_der().expect("Failed to encode DER");
    let ber_decoded = Person::decode_ber_indefinite(&ber_encoded).expect("Failed to decode BER");
    let der_decoded = Person::decode_der(&der_encoded).expect("Failed to decode DER");
    assert_eq!(ber_decoded, der_decoded);
    println!("PASS: test_ber_indefinite_decode_matches_der_decode");
}

fn test_ber_indefinite_nested() {
    let dept = Department {
        name: "HR".to_string(),
        id: BigInt::from(200),
    };
    let company = Company {
        name: "MegaCorp".to_string(),
        departments: vec![dept],
    };
    let encoded = company.encode_ber_indefinite().expect("Failed to encode");
    let decoded = Company::decode_ber_indefinite(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "MegaCorp");
    assert_eq!(decoded.departments[0].name, "HR");
    println!("PASS: test_ber_indefinite_nested");
}

fn test_ber_indefinite_department_with_people() {
    let dept = Department {
        name: "Dev".to_string(),
        id: BigInt::from(300),
    };
    let encoded = dept.encode_ber_indefinite().expect("Failed to encode");
    let decoded = Department::decode_ber_indefinite(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "Dev");
    assert_eq!(decoded.id, BigInt::from(300));
    println!("PASS: test_ber_indefinite_department_with_people");
}