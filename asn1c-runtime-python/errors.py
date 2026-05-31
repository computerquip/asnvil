"""BER encoding error types."""


class AsnError(Exception):
    """Base ASN.1 error."""
    pass


class UnexpectedTagError(AsnError):
    """Expected tag not found."""
    pass


class InvalidLengthError(AsnError):
    """Invalid length encoding."""
    pass


class InvalidIntegerEncodingError(AsnError):
    """Invalid INTEGER encoding."""
    pass


class TruncatedInputError(AsnError):
    """Input data truncated."""
    pass


class ConstraintViolationError(AsnError):
    """Constraint violation."""
    pass


class UnknownChoiceVariantError(AsnError):
    """Unknown CHOICE variant."""
    pass


class NonMinimalLengthError(AsnError):
    """Non-minimal length encoding (DER-specific)."""
    pass


class IndefiniteLengthNotAllowedError(AsnError):
    """Indefinite length not allowed in DER."""
    pass


class SetNotCanonicalError(AsnError):
    """SET not in canonical order (DER-specific)."""
    pass
