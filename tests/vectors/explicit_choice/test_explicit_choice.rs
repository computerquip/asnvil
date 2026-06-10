//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./TestModule.rs"]
mod generated;

use generated::{Entity, Person, Department};
use num_bigint::BigInt;

#[test]
fn test_entity_person_roundtrip() {
    let person = Person {
        name: "Alice".to_string(),
        age: BigInt::from(30),
        active: Some(true),
    };
    
    let entity = Entity::Person(person);
    let encoded = entity.encode_der().expect("Failed to encode");
    let decoded = Entity::decode_der(&encoded).expect("Failed to decode");
    
    match decoded {
        Entity::Person(p) => {
            assert_eq!(p.name, "Alice");
            assert_eq!(p.age, BigInt::from(30));
            assert_eq!(p.active, Some(true));
        }
        _ => panic!("Expected Person variant"),
    }
}

#[test]
fn test_entity_department_roundtrip() {
    let dept = Department {
        deptName: "Engineering".to_string(),
        code: BigInt::from(101),
        location: "Building A".to_string(),
    };
    
    let entity = Entity::Department(dept);
    let encoded = entity.encode_der().expect("Failed to encode");
    let decoded = Entity::decode_der(&encoded).expect("Failed to decode");
    
    match decoded {
        Entity::Department(d) => {
            assert_eq!(d.deptName, "Engineering");
            assert_eq!(d.code, BigInt::from(101));
            assert_eq!(d.location, "Building A".to_string());
        }
        _ => panic!("Expected Department variant"),
    }
}