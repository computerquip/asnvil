//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! num-bigint = "0.4"
//! ```

#[path = "./SNMPv2MIB.rs"]
mod generated;

use generated::{GetRequest, GetResponse, SetRequest, GetBulkRequest, VarBindList, VarBind};
use num_bigint::BigInt;
use asnvil_runtime_rust::ObjectIdentifier;

#[test]
fn test_get_request_roundtrip() {
    let req = GetRequest {
        requestID: BigInt::from(123),
        errorStatus: BigInt::from(0),
        errorIndex: BigInt::from(0),
        variableBindings: VarBindList(vec![]),
    };
    let encoded = req.encode_der().expect("Failed to encode");
    let decoded = GetRequest::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.requestID, BigInt::from(123));
}

#[test]
fn test_get_response_roundtrip() {
    let resp = GetResponse {
        requestID: BigInt::from(123),
        errorStatus: BigInt::from(0),
        errorIndex: BigInt::from(0),
        variableBindings: VarBindList(vec![]),
    };
    let encoded = resp.encode_der().expect("Failed to encode");
    let decoded = GetResponse::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.requestID, BigInt::from(123));
}

#[test]
fn test_set_request_roundtrip() {
    let req = SetRequest {
        requestID: BigInt::from(456),
        errorStatus: BigInt::from(0),
        errorIndex: BigInt::from(0),
        variableBindings: VarBindList(vec![]),
    };
    let encoded = req.encode_der().expect("Failed to encode");
    let decoded = SetRequest::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.requestID, BigInt::from(456));
}

#[test]
fn test_get_bulk_request_roundtrip() {
    let req = GetBulkRequest {
        requestID: BigInt::from(789),
        nonRepeaters: BigInt::from(0),
        maxRepetitions: BigInt::from(10),
        variableBindings: VarBindList(vec![]),
    };
    let encoded = req.encode_der().expect("Failed to encode");
    let decoded = GetBulkRequest::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.requestID, BigInt::from(789));
}

#[test]
fn test_var_bind_list_roundtrip() {
    let vb = VarBind {
        name: ObjectIdentifier::new(vec![1, 3, 6, 1, 2, 1, 1, 1, 0]).unwrap(),
        value: Some(vec![1, 2, 3]),
    };
    let list = VarBindList(vec![vb]);
    let encoded = list.encode_der().expect("Failed to encode");
    let decoded = VarBindList::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.0.len(), 1);
}

#[test]
fn test_var_bind_roundtrip() {
    let vb = VarBind {
        name: ObjectIdentifier::new(vec![1, 3, 6, 1, 2, 1, 1, 2, 0]).unwrap(),
        value: Some(vec![4, 5, 6]),
    };
    let encoded = vb.encode_der().expect("Failed to encode");
    let decoded = VarBind::decode_der(&encoded).expect("Failed to decode");
    assert_eq!(decoded.name, ObjectIdentifier::new(vec![1, 3, 6, 1, 2, 1, 1, 2, 0]).unwrap());
}

#[test]
fn test_snmp_syntax_integer() {
    let val = BigInt::from(42);
    assert_eq!(val, BigInt::from(42));
}

#[test]
fn test_snmp_syntax_string() {
    let val = "test string".to_string();
    assert_eq!(val, "test string");
}

#[test]
fn test_snmp_syntax_oid() {
    let oid = ObjectIdentifier::new(vec![1, 3, 6, 1, 2, 1, 1, 3, 0]).unwrap();
    let encoded = oid.encode().expect("Failed to encode");
    let (decoded, _) = ObjectIdentifier::decode(&encoded).expect("Failed to decode");
    assert_eq!(decoded, oid);
}