//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./PKIX1Explicit.rs"]
mod generated;

use generated::{AlgorithmIdentifier, Validity, Name, AttributeTypeAndValue, RelativeDistinguishedName, RDNSequence};
use asnvil_runtime_rust::ObjectIdentifier;

#[test]
fn test_algorithm_identifier_roundtrip() {
    let sha256_oid = ObjectIdentifier::new(vec![2, 16, 840, 1, 101, 3, 4, 2, 1]).unwrap();
    let alg = AlgorithmIdentifier {
        algorithm: sha256_oid.clone(),
        parameters: None,
    };
    let encoded = alg.encode_der().expect("Failed to encode");
    let decoded = AlgorithmIdentifier::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.algorithm, sha256_oid, "algorithm mismatch");
    assert!(decoded.parameters.is_none(), "parameters should be None");
}

#[test]
fn test_validity_roundtrip() {
    let v = Validity {
        notBefore: "20240115120000Z".to_string(),
        notAfter: "20250115120000Z".to_string(),
    };
    let encoded = v.encode_der().expect("Failed to encode");
    let decoded = Validity::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.notBefore, v.notBefore, "notBefore mismatch");
    assert_eq!(decoded.notAfter, v.notAfter, "notAfter mismatch");
}

#[test]
fn test_name_roundtrip() {
    let cn_attr = AttributeTypeAndValue {
        r#type: ObjectIdentifier::new(vec![2, 5, 4, 3]).unwrap(),
        value: b"Example CA".to_vec(),
    };
    let rdn: RelativeDistinguishedName = vec![cn_attr.clone()];
    let rdn_seq: RDNSequence = vec![rdn];
    let name = Name { r#type: rdn_seq };
    
    let encoded = name.encode_der().expect("Failed to encode");
    let decoded = Name::decode_der(&encoded).expect("Failed to decode");
    
    assert_eq!(decoded.r#type.len(), 1);
    assert_eq!(decoded.r#type[0].len(), 1);
    assert_eq!(decoded.r#type[0][0].r#type, cn_attr.r#type);
    assert_eq!(decoded.r#type[0][0].value, cn_attr.value);
}