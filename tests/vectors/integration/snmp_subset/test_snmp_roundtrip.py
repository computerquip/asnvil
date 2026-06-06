import sys
import os
# The integration test runner sets PYTHONPATH to the compiled output dir.
# When running standalone, set INTEG_OUTPUT_DIR to the output directory.
_integ_output = os.environ.get("INTEG_OUTPUT_DIR")
if _integ_output:
    sys.path.insert(0, _integ_output)

from SNMPv2MIB import (
    SNMPv2Message, SNMPv2Messages, GetRequest, GetResponse,
    SetRequest, GetBulkRequest, VarBindList, VarBind,
    SnmpSyntax, Counter32, Gauge32, TimeTicks,
)
from asnvil_runtime import ObjectIdentifier


def test_get_request_roundtrip():
    """Test GetRequest with VarBindList."""
    vb = VarBind(
        name=ObjectIdentifier([1, 3, 6, 1, 2, 1, 1, 1, 0]),
        value=SnmpSyntax(integerValue=42),
    )
    req = GetRequest(
        requestId=1234,
        errorStatus=0,
        errorIndex=0,
        variableBindings=VarBindList([vb]),
    )
    encoded = req.encode_der()
    decoded = GetRequest.decode_der(encoded)
    assert decoded.requestId == 1234
    assert decoded.errorStatus == 0
    assert len(decoded.variableBindings) == 1
    assert decoded.variableBindings[0].name == vb.name
    print("PASS: GetRequest roundtrip")


def test_get_response_roundtrip():
    """Test GetResponse with errorStatus/errorIndex."""
    vb = VarBind(
        name=ObjectIdentifier([1, 3, 6, 1, 2, 1, 1, 5, 0]),
        value=SnmpSyntax(stringValue=b'router-01'),
    )
    resp = GetResponse(
        requestId=5678,
        errorStatus=2,
        errorIndex=1,
        variableBindings=VarBindList([vb]),
    )
    encoded = resp.encode_der()
    decoded = GetResponse.decode_der(encoded)
    assert decoded.requestId == 5678
    assert decoded.errorStatus == 2
    assert decoded.errorIndex == 1
    print("PASS: GetResponse roundtrip")


def test_set_request_roundtrip():
    """Test SetRequest with VarBind."""
    vb = VarBind(
        name=ObjectIdentifier([1, 3, 6, 1, 2, 1, 1, 6, 0]),
        value=SnmpSyntax(stringValue=b'dc1-rack3'),
    )
    req = SetRequest(
        requestId=9999,
        errorStatus=0,
        errorIndex=0,
        variableBindings=VarBindList([vb]),
    )
    encoded = req.encode_der()
    decoded = SetRequest.decode_der(encoded)
    assert decoded.requestId == 9999
    assert decoded.variableBindings[0].value.stringValue == b'dc1-rack3'
    print("PASS: SetRequest roundtrip")


def test_get_bulk_request_roundtrip():
    """Test GetBulkRequest with nonRepeaters/maxRepetitions."""
    req = GetBulkRequest(
        requestId=4321,
        nonRepeaters=1,
        maxRepetitions=10,
        variableBindings=VarBindList([]),
    )
    encoded = req.encode_der()
    decoded = GetBulkRequest.decode_der(encoded)
    assert decoded.requestId == 4321
    assert decoded.nonRepeaters == 1
    assert decoded.maxRepetitions == 10
    print("PASS: GetBulkRequest roundtrip")


def test_var_bind_list_roundtrip():
    """Test SEQUENCE OF VarBind."""
    vb1 = VarBind(
        name=ObjectIdentifier([1, 3, 6, 1, 2, 1, 1, 1, 0]),
        value=SnmpSyntax(integerValue=100),
    )
    vb2 = VarBind(
        name=ObjectIdentifier([1, 3, 6, 1, 2, 1, 1, 5, 0]),
        value=SnmpSyntax(stringValue=b'switch-01'),
    )
    vbl = VarBindList([vb1, vb2])
    encoded = vbl.encode_der()
    decoded = VarBindList.decode_der(encoded)
    assert len(decoded) == 2
    assert decoded[0].value.integerValue == 100
    assert decoded[1].value.stringValue == b'switch-01'
    print("PASS: VarBindList roundtrip")


def test_var_bind_roundtrip():
    """Test VarBind with ObjectIdentifier + SnmpSyntax."""
    vb = VarBind(
        name=ObjectIdentifier([1, 3, 6, 1, 2, 1, 1, 3, 0]),
        value=SnmpSyntax(integerValue=99999),
    )
    encoded = vb.encode_der()
    decoded = VarBind.decode_der(encoded)
    assert decoded.name == vb.name
    assert decoded.value.integerValue == 99999
    print("PASS: VarBind roundtrip")


def test_snmp_syntax_integer():
    """Test SnmpSyntax CHOICE with integerValue."""
    syntax = SnmpSyntax(integerValue=-42)
    encoded = syntax.encode_der()
    decoded = SnmpSyntax.decode_der(encoded)
    assert decoded.integerValue == -42
    print("PASS: SnmpSyntax integerValue roundtrip")


def test_snmp_syntax_string():
    """Test SnmpSyntax CHOICE with stringValue."""
    syntax = SnmpSyntax(stringValue=b'Hello SNMP World')
    encoded = syntax.encode_der()
    decoded = SnmpSyntax.decode_der(encoded)
    assert decoded.stringValue == b'Hello SNMP World'
    print("PASS: SnmpSyntax stringValue roundtrip")


def test_snmp_syntax_oid():
    """Test SnmpSyntax CHOICE with oidValue."""
    oid = ObjectIdentifier([1, 3, 6, 1, 2, 1, 1, 1])
    syntax = SnmpSyntax(oidValue=oid)
    encoded = syntax.encode_der()
    decoded = SnmpSyntax.decode_der(encoded)
    assert decoded.oidValue == oid
    print("PASS: SnmpSyntax oidValue roundtrip")


if __name__ == "__main__":
    test_get_request_roundtrip()
    test_get_response_roundtrip()
    test_set_request_roundtrip()
    test_get_bulk_request_roundtrip()
    test_var_bind_list_roundtrip()
    test_var_bind_roundtrip()
    test_snmp_syntax_integer()
    test_snmp_syntax_string()
    test_snmp_syntax_oid()
    print("\nAll SNMP integration tests passed!")
