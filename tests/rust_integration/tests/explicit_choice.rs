// Integration test for explicit_choice

mod generated {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/TestModule.rs"));
}

use generated::{Person, Department, Entity, Container, MixedChoice};
use asnvil_runtime_rust::{DerDecoder, TagClass};

#[test]
fn test_entity_person_roundtrip() {
    let person = Person {
        name: "Alice".to_string(),
        age: num_bigint::BigInt::from(30),
        active: Some(true),
    };
    let entity = Entity::Person(person);
    let encoded = entity.encode_der().unwrap();
    println!("Encoded: {:02x?}", encoded);
    
    // Manual decode to find the exact failure point
    let mut decoder = DerDecoder::new(&encoded);
    println!("1. Reading tag...");
    let (tag_class, tag_number, constructed) = decoder.read_tag().unwrap();
    println!("   Tag: {:?}, {}, {}", tag_class, tag_number, constructed);
    
    println!("2. Reading length...");
    let _len = decoder.read_length().unwrap();
    println!("   Length: {}", _len);
    
    println!("3. Reading content...");
    let _content = decoder.read_bytes(_len).unwrap();
    println!("   Content: {:02x?}", _content);
    
    println!("4. Calling Person::decode_der...");
    let person_decoded = Person::decode_der(_content);
    println!("   Result: {:?}", person_decoded);
    
    let decoded = Entity::decode_der(&encoded).unwrap();
    
    match decoded {
        Entity::Person(p) => {
            assert_eq!(p.name, "Alice");
            assert_eq!(p.age, num_bigint::BigInt::from(30));
            assert_eq!(p.active, Some(true));
        }
        _ => panic!("Expected Person variant"),
    }
    
    // Verify the outer tag is CONTEXT 0
    let mut dec = DerDecoder::new(&encoded);
    let (tag_class, tag_number, constructed) = dec.read_tag().unwrap();
    assert_eq!(tag_class, TagClass::Context);
    assert_eq!(tag_number, 0);
    assert_eq!(constructed, true);
}

#[test]
fn test_entity_department_roundtrip() {
    let dept = Department {
        deptName: "Engineering".to_string(),
        code: num_bigint::BigInt::from(100),
        location: "Building A".to_string(),
    };
    let entity = Entity::Department(dept);
    let encoded = entity.encode_der().unwrap();
    let decoded = Entity::decode_der(&encoded).unwrap();
    
    match decoded {
        Entity::Department(d) => {
            assert_eq!(d.deptName, "Engineering");
            assert_eq!(d.code, num_bigint::BigInt::from(100));
            assert_eq!(d.location, "Building A");
        }
        _ => panic!("Expected Department variant"),
    }
}

#[test]
fn test_mixed_choice_integer() {
    let choice = MixedChoice::Count(num_bigint::BigInt::from(42));
    let encoded = choice.encode_der().unwrap();
    let decoded = MixedChoice::decode_der(&encoded).unwrap();
    
    match decoded {
        MixedChoice::Count(c) => {
            assert_eq!(c, num_bigint::BigInt::from(42));
        }
        _ => panic!("Expected Count variant"),
    }
}
