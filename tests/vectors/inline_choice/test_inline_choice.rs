//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./TestModule.rs"]
mod generated;

use generated::{Person, ContactInfo};
use num_bigint::BigInt;

fn main() {
    test_person_with_email();
    test_person_with_phone();
    test_person_no_contact();
    test_der_roundtrip_email();
    test_ber_indefinite_roundtrip();
    test_choice_encode_ber();
    test_choice_decode_der();
    test_choice_encode_der();
    test_choice_ber_indefinite();
    println!("\nAll Inline Choice integration tests passed!");
}

fn test_person_with_email() {
    let contact = ContactInfo::email("test@example.com".to_string());
    let person = Person {
        name: "Alice".to_string(),
        age: BigInt::from(30),
        contact: Some(contact),
    };
    let encoded = person.encode_der().expect("Failed to encode");
    let decoded = Person::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "Alice");
    match decoded.contact {
        Some(ContactInfo::email(e)) => assert_eq!(e, "test@example.com"),
        _ => panic!("Expected email"),
    }
    println!("PASS: test_person_with_email");
}

fn test_person_with_phone() {
    let contact = ContactInfo::phone("555-1234".to_string());
    let person = Person {
        name: "Bob".to_string(),
        age: BigInt::from(25),
        contact: Some(contact),
    };
    let encoded = person.encode_der().expect("Failed to encode");
    let decoded = Person::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "Bob");
    match decoded.contact {
        Some(ContactInfo::phone(p)) => assert_eq!(p, "555-1234"),
        _ => panic!("Expected phone"),
    }
    println!("PASS: test_person_with_phone");
}

fn test_person_no_contact() {
    let person = Person {
        name: "Charlie".to_string(),
        age: BigInt::from(40),
        contact: None,
    };
    let encoded = person.encode_der().expect("Failed to encode");
    let decoded = Person::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, "Charlie");
    assert!(decoded.contact.is_none());
    println!("PASS: test_person_no_contact");
}

fn test_der_roundtrip_email() {
    let contact = ContactInfo::email("der@test.com".to_string());
    let encoded = contact.encode_der().expect("Failed to encode");
    let decoded = ContactInfo::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        ContactInfo::email(e) => assert_eq!(e, "der@test.com"),
        _ => panic!("Expected email"),
    }
    println!("PASS: test_der_roundtrip_email");
}

fn test_ber_indefinite_roundtrip() {
    let contact = ContactInfo::phone("555-9999".to_string());
    let encoded = contact.encode_ber_indefinite().expect("Failed to encode");
    let decoded = ContactInfo::decode_ber_indefinite(&encoded).expect("Failed to decode");
    match decoded {
        ContactInfo::phone(p) => assert_eq!(p, "555-9999"),
        _ => panic!("Expected phone"),
    }
    println!("PASS: test_ber_indefinite_roundtrip");
}

fn test_choice_encode_ber() {
    let contact = ContactInfo::email("ber@test.com".to_string());
    let encoded = contact.encode_ber().expect("Failed to encode");
    assert!(!encoded.is_empty());
    println!("PASS: test_choice_encode_ber");
}

fn test_choice_decode_der() {
    let contact = ContactInfo::phone("123-4567".to_string());
    let encoded = contact.encode_der().expect("Failed to encode");
    let decoded = ContactInfo::decode_der(&encoded).expect("Failed to decode");
    match decoded {
        ContactInfo::phone(p) => assert_eq!(p, "123-4567"),
        _ => panic!("Expected phone"),
    }
    println!("PASS: test_choice_decode_der");
}

fn test_choice_encode_der() {
    let contact = ContactInfo::email("der2@test.com".to_string());
    let encoded = contact.encode_der().expect("Failed to encode");
    assert!(!encoded.is_empty());
    println!("PASS: test_choice_encode_der");
}

fn test_choice_ber_indefinite() {
    let contact = ContactInfo::phone("999-0000".to_string());
    let encoded = contact.encode_ber_indefinite().expect("Failed to encode");
    assert!(!encoded.is_empty());
    println!("PASS: test_choice_ber_indefinite");
}