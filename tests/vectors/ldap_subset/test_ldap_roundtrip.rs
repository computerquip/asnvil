//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./LDAPv3.rs"]
mod generated;

use generated::{
    AttributeValueAssertion, PartialAttribute, LDAPResult, SearchResultEntry, BindRequest,
    BindResponse, SearchRequest, LDAPMessage, ProtocolOp,
};
use num_bigint::BigInt;

fn main() {
    test_attribute_value_assertion_roundtrip();
    test_partial_attribute_roundtrip();
    test_ldap_result_roundtrip();
    test_search_result_entry_roundtrip();
    test_bind_request_roundtrip();
    test_bind_response_roundtrip();
    test_search_request_roundtrip();
    test_ldap_message_roundtrip();
    test_partial_attribute_list_roundtrip();
    println!("\nAll LDAP Subset integration tests passed!");
}

fn test_attribute_value_assertion_roundtrip() {
    let ava = AttributeValueAssertion {
        attributeDesc: "cn".to_string(),
        assertionValue: b"test".to_vec(),
    };
    let encoded = ava.encode_der().expect("Failed to encode");
    let decoded = AttributeValueAssertion::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.attributeDesc, "cn");
    assert_eq!(decoded.assertionValue, b"test".to_vec());
    println!("PASS: test_attribute_value_assertion_roundtrip");
}

fn test_partial_attribute_roundtrip() {
    let pa = PartialAttribute {
        r#type: "objectClass".to_string(),
        vals: vec![b"top".to_vec(), b"person".to_vec()],
    };
    let encoded = pa.encode_der().expect("Failed to encode");
    let decoded = PartialAttribute::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.r#type, "objectClass");
    assert_eq!(decoded.vals.len(), 2);
    println!("PASS: test_partial_attribute_roundtrip");
}

fn test_ldap_result_roundtrip() {
    let result = LDAPResult {
        resultCode: BigInt::from(0),
        matchedDN: "".to_string(),
        errorMessage: "".to_string(),
        referral: None,
    };
    let encoded = result.encode_der().expect("Failed to encode");
    let decoded = LDAPResult::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.resultCode, BigInt::from(0));
    println!("PASS: test_ldap_result_roundtrip");
}

fn test_search_result_entry_roundtrip() {
    let entry = SearchResultEntry {
        objectName: "cn=test,dc=example,dc=com".to_string(),
        attributes: vec![],
    };
    let encoded = entry.encode_der().expect("Failed to encode");
    let decoded = SearchResultEntry::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.objectName, "cn=test,dc=example,dc=com");
    println!("PASS: test_search_result_entry_roundtrip");
}

fn test_bind_request_roundtrip() {
    let req = BindRequest {
        version: BigInt::from(3),
        name: "cn=admin,dc=example,dc=com".to_string(),
        authentication: b"secret".to_vec(),
    };
    let encoded = req.encode_der().expect("Failed to encode");
    let decoded = BindRequest::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.version, BigInt::from(3));
    println!("PASS: test_bind_request_roundtrip");
}

fn test_bind_response_roundtrip() {
    let resp = BindResponse {
        resultCode: BigInt::from(0),
        matchedDN: "".to_string(),
        errorMessage: "".to_string(),
        referral: None,
        serverSaslCreds: None,
    };
    let encoded = resp.encode_der().expect("Failed to encode");
    let decoded = BindResponse::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.resultCode, BigInt::from(0));
    println!("PASS: test_bind_response_roundtrip");
}

fn test_search_request_roundtrip() {
    let req = SearchRequest {
        baseObject: "dc=example,dc=com".to_string(),
        scope: BigInt::from(2),
        derefAliases: BigInt::from(0),
        sizeLimit: BigInt::from(0),
        timeLimit: BigInt::from(0),
        typesOnly: false,
        filter: b"(\x63\x6e=*)" .to_vec(), // Simplified filter
        attributes: vec![],
    };
    let encoded = req.encode_der().expect("Failed to encode");
    let decoded = SearchRequest::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.baseObject, "dc=example,dc=com");
    println!("PASS: test_search_request_roundtrip");
}

fn test_ldap_message_roundtrip() {
    let req = BindRequest {
        version: BigInt::from(3),
        name: "cn=admin".to_string(),
        authentication: b"secret".to_vec(),
    };
    let msg = LDAPMessage {
        messageID: BigInt::from(1),
        protocolOp: ProtocolOp::bindRequest(req),
        controls: None,
    };
    let encoded = msg.encode_der().expect("Failed to encode");
    let decoded = LDAPMessage::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.messageID, BigInt::from(1));
    println!("PASS: test_ldap_message_roundtrip");
}

fn test_partial_attribute_list_roundtrip() {
    let pa = PartialAttribute {
        r#type: "mail".to_string(),
        vals: vec![b"test@example.com".to_vec()],
    };
    let encoded = pa.encode_der().expect("Failed to encode");
    let decoded = PartialAttribute::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.r#type, "mail");
    println!("PASS: test_partial_attribute_list_roundtrip");
}