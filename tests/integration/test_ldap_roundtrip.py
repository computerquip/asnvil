import sys
sys.path.insert(0, '/tmp/asn1c-integration-test/ldap')

from LDAPv3 import (
    AttributeValueAssertion, PartialAttribute, LDAPResult,
    SearchResultEntry, BindRequest, BindResponse, SearchRequest,
    LDAPMessage, PartialAttributeList,
)
from asn1c_runtime import ObjectIdentifier


def test_attribute_value_assertion_roundtrip():
    """Test AttributeValueAssertion encode/decode."""
    ava = AttributeValueAssertion(
        attributeDesc=b'cn',
        assertionValue=b'John Doe',
    )
    encoded = ava.encode_der()
    decoded = AttributeValueAssertion.decode_der(encoded)
    assert decoded.attributeDesc == b'cn', f"attributeDesc mismatch"
    assert decoded.assertionValue == b'John Doe', f"assertionValue mismatch"
    print("PASS: AttributeValueAssertion roundtrip")


def test_partial_attribute_roundtrip():
    """Test PartialAttribute encode/decode."""
    attr = PartialAttribute(
        typeAttr=b'cn',
        vals=[b'John Doe', b'J. Doe'],
    )
    encoded = attr.encode_der()
    decoded = PartialAttribute.decode_der(encoded)
    assert decoded.typeAttr == b'cn', f"typeAttr mismatch"
    assert len(decoded.vals) == 2, f"vals count mismatch"
    print("PASS: PartialAttribute roundtrip")


def test_ldap_result_roundtrip():
    """Test LDAPResult encode/decode."""
    result = LDAPResult(
        resultCode=0,  # success
        matchedDN=b'',
        diagnosticMessage=b'',
    )
    encoded = result.encode_der()
    decoded = LDAPResult.decode_der(encoded)
    assert decoded.resultCode == 0, f"resultCode mismatch"
    assert decoded.matchedDN == b'', f"matchedDN mismatch"
    assert decoded.diagnosticMessage == b'', f"diagnosticMessage mismatch"
    print("PASS: LDAPResult roundtrip")


def test_search_result_entry_roundtrip():
    """Test SearchResultEntry encode/decode."""
    attr = PartialAttribute(
        typeAttr=b'cn',
        vals=[b'John Doe'],
    )
    entry = SearchResultEntry(
        objectName=b'cn=John Doe,dc=example,dc=com',
        attributes=[attr],
    )
    encoded = entry.encode_der()
    decoded = SearchResultEntry.decode_der(encoded)
    assert decoded.objectName == b'cn=John Doe,dc=example,dc=com'
    assert len(decoded.attributes) == 1
    assert decoded.attributes[0].typeAttr == b'cn'
    print("PASS: SearchResultEntry roundtrip")


def test_bind_request_roundtrip():
    """Test BindRequest encode/decode."""
    req = BindRequest(
        version=3,
        name=b'cn=admin,dc=example,dc=com',
        authentication=b'secret',
    )
    encoded = req.encode_der()
    decoded = BindRequest.decode_der(encoded)
    assert decoded.version == 3
    assert decoded.name == b'cn=admin,dc=example,dc=com'
    assert decoded.authentication == b'secret'
    print("PASS: BindRequest roundtrip")


def test_bind_response_roundtrip():
    """Test BindResponse encode/decode."""
    resp = BindResponse(
        resultCode=0,
        matchedDN=b'',
        diagnosticMessage=b'',
    )
    encoded = resp.encode_der()
    decoded = BindResponse.decode_der(encoded)
    assert decoded.resultCode == 0
    print("PASS: BindResponse roundtrip")


def test_search_request_roundtrip():
    """Test SearchRequest encode/decode."""
    req = SearchRequest(
        baseObject=b'dc=example,dc=com',
        scope=2,  # wholeSubtree
        derefAliases=0,  # neverDerefAliases
        sizeLimit=100,
        timeLimit=30,
        typesOnly=False,
        filter=b'(cn=*)',
        attributes=[b'cn', b'mail'],
    )
    encoded = req.encode_der()
    decoded = SearchRequest.decode_der(encoded)
    assert decoded.baseObject == b'dc=example,dc=com'
    assert decoded.scope == 2
    assert decoded.sizeLimit == 100
    assert len(decoded.attributes) == 2
    print("PASS: SearchRequest roundtrip")


def test_ldap_message_roundtrip():
    """Test LDAPMessage encode/decode."""
    msg = LDAPMessage(
        messageID=1,
        protocolOp=b'\x01\x01\x00',  # dummy protocolOp
    )
    encoded = msg.encode_der()
    decoded = LDAPMessage.decode_der(encoded)
    assert decoded.messageID == 1
    print("PASS: LDAPMessage roundtrip")


def test_partial_attribute_list_roundtrip():
    """Test PartialAttributeList encode/decode."""
    attr1 = PartialAttribute(typeAttr=b'cn', vals=[b'John Doe'])
    attr2 = PartialAttribute(typeAttr=b'mail', vals=[b'john@example.com'])
    attr_list = PartialAttributeList([attr1, attr2])
    encoded = attr_list.encode_der()
    decoded = PartialAttributeList.decode_der(encoded)
    assert len(decoded) == 2
    assert decoded[0].typeAttr == b'cn'
    assert decoded[1].typeAttr == b'mail'
    print("PASS: PartialAttributeList roundtrip")


if __name__ == "__main__":
    test_attribute_value_assertion_roundtrip()
    test_partial_attribute_roundtrip()
    test_ldap_result_roundtrip()
    test_search_result_entry_roundtrip()
    test_bind_request_roundtrip()
    test_bind_response_roundtrip()
    test_search_request_roundtrip()
    test_ldap_message_roundtrip()
    test_partial_attribute_list_roundtrip()
    print("\nAll LDAP integration tests passed!")
