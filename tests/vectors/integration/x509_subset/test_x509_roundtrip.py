import sys
import os
# The integration test runner sets PYTHONPATH to the compiled output dir.
# When running standalone, set INTEG_OUTPUT_DIR to the output directory.
_integ_output = os.environ.get("INTEG_OUTPUT_DIR")
if _integ_output:
    sys.path.insert(0, _integ_output)

from PKIX1Explicit import (
    Certificate, TBSCertificate, AlgorithmIdentifier, Validity,
    Name, SubjectPublicKeyInfo, Version, TBSCertList,
    CertificateList, Extension, Extensions,
    AttributeTypeAndValue, RelativeDistinguishedName, RDNSequence,
)
from asnvil_runtime import ObjectIdentifier, BitString
from datetime import datetime


def test_algorithm_identifier_roundtrip():
    """Test AlgorithmIdentifier encode/decode."""
    sha256_oid = ObjectIdentifier([2, 16, 840, 1, 101, 3, 4, 2, 1])
    alg = AlgorithmIdentifier(algorithm=sha256_oid, parameters=None)
    encoded = alg.encode_der()
    decoded = AlgorithmIdentifier.decode_der(encoded)
    assert decoded.algorithm == sha256_oid, f"algorithm mismatch: {decoded.algorithm} != {sha256_oid}"
    assert decoded.parameters is None, f"parameters should be None, got {decoded.parameters}"
    print("PASS: AlgorithmIdentifier roundtrip")


def test_validity_roundtrip():
    """Test Validity encode/decode."""
    v = Validity(
        notBefore=datetime(2024, 1, 15, 12, 0, 0),
        notAfter=datetime(2025, 1, 15, 12, 0, 0),
    )
    encoded = v.encode_der()
    decoded = Validity.decode_der(encoded)
    assert decoded.notBefore == v.notBefore, f"notBefore mismatch"
    assert decoded.notAfter == v.notAfter, f"notAfter mismatch"
    print("PASS: Validity roundtrip")


def test_name_roundtrip():
    """Test Name encode/decode."""
    cn_attr = AttributeTypeAndValue(
        type=ObjectIdentifier([2, 5, 4, 3]),
        value=b'Example CA',
    )
    rdn = RelativeDistinguishedName([cn_attr])
    rdn_seq = RDNSequence([rdn])
    name = Name(rdnSequence=rdn_seq)
    encoded = name.encode_der()
    decoded = Name.decode_der(encoded)
    assert len(decoded.rdnSequence) == 1
    assert len(decoded.rdnSequence[0]) == 1
    assert decoded.rdnSequence[0][0].type == cn_attr.type
    assert decoded.rdnSequence[0][0].value == cn_attr.value
    print("PASS: Name roundtrip")


def test_extension_roundtrip():
    """Test Extension encode/decode."""
    ext = Extension(
        extnID=ObjectIdentifier([2, 5, 29, 15]),  # keyUsage
        extnValue=b'\x03\x02\x07\x80',
        critical=True,
    )
    encoded = ext.encode_der()
    decoded = Extension.decode_der(encoded)
    assert decoded.extnID == ext.extnID
    assert decoded.extnValue == ext.extnValue
    assert decoded.critical == ext.critical
    print("PASS: Extension roundtrip")


def test_extensions_roundtrip():
    """Test Extensions (SEQUENCE OF Extension) encode/decode."""
    ext1 = Extension(
        extnID=ObjectIdentifier([2, 5, 29, 15]),
        extnValue=b'\x03\x02\x07\x80',
        critical=True,
    )
    ext2 = Extension(
        extnID=ObjectIdentifier([2, 5, 29, 19]),  # basicConstraints
        extnValue=b'\x30\x03\x01\x01\xff',
        critical=True,
    )
    exts = Extensions([ext1, ext2])
    encoded = exts.encode_der()
    decoded = Extensions.decode_der(encoded)
    assert len(decoded) == 2
    assert decoded[0].extnID == ext1.extnID
    assert decoded[1].extnID == ext2.extnID
    print("PASS: Extensions roundtrip")


def test_subject_public_key_info_roundtrip():
    """Test SubjectPublicKeyInfo encode/decode."""
    sha256_oid = ObjectIdentifier([2, 16, 840, 1, 101, 3, 4, 2, 1])
    alg = AlgorithmIdentifier(algorithm=sha256_oid, parameters=None)
    pub_key = SubjectPublicKeyInfo(
        algorithm=alg,
        subjectPublicKey=BitString(b'\x30\x82\x01\x22\x30\x0d', unused_bits=0),
    )
    encoded = pub_key.encode_der()
    decoded = SubjectPublicKeyInfo.decode_der(encoded)
    assert decoded.algorithm.algorithm == sha256_oid
    assert decoded.subjectPublicKey.data == pub_key.subjectPublicKey.data
    print("PASS: SubjectPublicKeyInfo roundtrip")


def test_tbs_certificate_roundtrip():
    """Test TBSCertificate encode/decode (core X.509 structure)."""
    sha256_oid = ObjectIdentifier([1, 2, 840, 113549, 1, 1, 11])  # sha256WithRSAEncryption
    alg = AlgorithmIdentifier(algorithm=sha256_oid, parameters=None)

    cn_attr = AttributeTypeAndValue(
        type=ObjectIdentifier([2, 5, 4, 3]),
        value=b'Test CA',
    )
    rdn = RelativeDistinguishedName([cn_attr])
    name = Name(rdnSequence=RDNSequence([rdn]))

    validity = Validity(
        notBefore=datetime(2024, 1, 1, 0, 0, 0),
        notAfter=datetime(2025, 1, 1, 0, 0, 0),
    )

    pub_key = SubjectPublicKeyInfo(
        algorithm=alg,
        subjectPublicKey=BitString(b'\x00' * 32, unused_bits=0),
    )

    tbs = TBSCertificate(
        version=Version(2),  # v3
        serialNumber=12345,
        signature=alg,
        issuer=name,
        validity=validity,
        subject=name,
        subjectPublicKeyInfo=pub_key,
    )

    encoded = tbs.encode_der()
    decoded = TBSCertificate.decode_der(encoded)
    assert decoded.serialNumber == 12345
    assert decoded.version == 2
    assert decoded.issuer.rdnSequence[0][0].value == b'Test CA'
    assert decoded.validity.notBefore == datetime(2024, 1, 1, 0, 0, 0)
    print("PASS: TBSCertificate roundtrip")


def test_certificate_roundtrip():
    """Test full Certificate encode/decode."""
    sha256_oid = ObjectIdentifier([1, 2, 840, 113549, 1, 1, 11])
    alg = AlgorithmIdentifier(algorithm=sha256_oid, parameters=None)

    cn_attr = AttributeTypeAndValue(
        type=ObjectIdentifier([2, 5, 4, 3]),
        value=b'Test CA',
    )
    rdn = RelativeDistinguishedName([cn_attr])
    name = Name(rdnSequence=RDNSequence([rdn]))

    validity = Validity(
        notBefore=datetime(2024, 1, 1, 0, 0, 0),
        notAfter=datetime(2025, 1, 1, 0, 0, 0),
    )

    pub_key = SubjectPublicKeyInfo(
        algorithm=alg,
        subjectPublicKey=BitString(b'\x00' * 32, unused_bits=0),
    )

    tbs = TBSCertificate(
        version=Version(2),
        serialNumber=12345,
        signature=alg,
        issuer=name,
        validity=validity,
        subject=name,
        subjectPublicKeyInfo=pub_key,
    )

    cert = Certificate(
        tbsCertificate=tbs,
        signatureAlgorithm=alg,
        signatureValue=BitString(b'\x01\x02\x03\x04\x05', unused_bits=0),
    )

    encoded = cert.encode_der()
    decoded = Certificate.decode_der(encoded)
    assert decoded.tbsCertificate.serialNumber == 12345
    assert decoded.signatureValue.data == b'\x01\x02\x03\x04\x05'
    print("PASS: Certificate roundtrip")


def test_ber_der_equivalence():
    """Test that DER encode/decode roundtrips correctly for Certificate."""
    sha256_oid = ObjectIdentifier([1, 2, 840, 113549, 1, 1, 11])
    alg = AlgorithmIdentifier(algorithm=sha256_oid, parameters=None)
    cn_attr = AttributeTypeAndValue(
        type=ObjectIdentifier([2, 5, 4, 3]),
        value=b'Test CA',
    )
    name = Name(rdnSequence=RDNSequence([RelativeDistinguishedName([cn_attr])]))
    validity = Validity(
        notBefore=datetime(2024, 1, 1, 0, 0, 0),
        notAfter=datetime(2025, 1, 1, 0, 0, 0),
    )
    pub_key = SubjectPublicKeyInfo(
        algorithm=alg,
        subjectPublicKey=BitString(b'\x00' * 32, unused_bits=0),
    )
    tbs = TBSCertificate(
        version=Version(2),
        serialNumber=12345,
        signature=alg,
        issuer=name,
        validity=validity,
        subject=name,
        subjectPublicKeyInfo=pub_key,
    )
    cert = Certificate(
        tbsCertificate=tbs,
        signatureAlgorithm=alg,
        signatureValue=BitString(b'\x01\x02\x03', unused_bits=0),
    )
    der_encoded = cert.encode_der()
    decoded = Certificate.decode_der(der_encoded)
    assert decoded.tbsCertificate.serialNumber == 12345
    assert decoded.signatureValue.data == b'\x01\x02\x03'
    assert decoded.tbsCertificate.version == 2
    print("PASS: Certificate DER roundtrip")


if __name__ == "__main__":
    test_algorithm_identifier_roundtrip()
    test_validity_roundtrip()
    test_name_roundtrip()
    test_extension_roundtrip()
    test_extensions_roundtrip()
    test_subject_public_key_info_roundtrip()
    test_tbs_certificate_roundtrip()
    test_certificate_roundtrip()
    test_ber_der_equivalence()
    print("\nAll X.509 integration tests passed!")
