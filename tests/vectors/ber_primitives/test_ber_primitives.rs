//! ```cargo
//! [dependencies]
//! asnvil-runtime-rust = { path = "__REPO_ROOT__/asnvil-runtime-rust" }
//! ```

use asnvil_runtime_rust::{TagClass, BerDecoder};

#[test]
fn test_ber_tag_decoding_universal() {
    let ber_bytes = [0x02, 0x01, 0x05];
    let mut decoder = BerDecoder::new(&ber_bytes);
    let (tag_class, tag_number, constructed) = decoder.read_tag().expect("Failed to read tag");
    assert_eq!(tag_class, TagClass::Universal);
    assert_eq!(tag_number, 2);
    assert!(!constructed);
}

#[test]
fn test_ber_tag_decoding_context() {
    let ber_bytes = [0x80, 0x00];
    let mut decoder = BerDecoder::new(&ber_bytes);
    let (tag_class, tag_number, constructed) = decoder.read_tag().expect("Failed to read tag");
    assert_eq!(tag_class, TagClass::Context);
    assert_eq!(tag_number, 0);
    assert!(!constructed);
}

#[test]
fn test_ber_tag_decoding_application() {
    let ber_bytes = [0x63, 0x00];
    let mut decoder = BerDecoder::new(&ber_bytes);
    let (tag_class, tag_number, constructed) = decoder.read_tag().expect("Failed to read tag");
    assert_eq!(tag_class, TagClass::Application);
    assert_eq!(tag_number, 3);
    assert!(!constructed);
}

#[test]
fn test_ber_tag_decoding_private() {
    let ber_bytes = [0xDF, 0x1B, 0x00];
    let mut decoder = BerDecoder::new(&ber_bytes);
    let (tag_class, tag_number, constructed) = decoder.read_tag().expect("Failed to read tag");
    assert_eq!(tag_class, TagClass::Private);
    assert_eq!(tag_number, 27);
    assert!(!constructed);
}

#[test]
fn test_ber_tag_decoding_integer() {
    let ber_bytes = [0x02, 0x01, 0x01];
    let mut decoder = BerDecoder::new(&ber_bytes);
    let (tag_class, tag_number, constructed) = decoder.read_tag().expect("Failed to read tag");
    assert_eq!(tag_class, TagClass::Universal);
    assert_eq!(tag_number, 2);
    assert!(!constructed);
    let val = decoder.read_integer().expect("Failed to read integer");
    assert_eq!(val, 1);
}