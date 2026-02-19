"""Seismic Python SDK -- web3.py extensions for the Seismic privacy-enabled EVM.

Public API
----------

**Types** (``seismic_web3._types``):
    :class:`Bytes32`, :class:`PrivateKey`, :class:`CompressedPublicKey`,
    :class:`EncryptionNonce`

**Chains** (``seismic_web3.chains``):
    :class:`ChainConfig`, :data:`SEISMIC_TESTNET`, :data:`SANVIL`,
    :func:`make_seismic_testnet`, :data:`SEISMIC_TX_TYPE`

**Transaction types** (``seismic_web3.transaction_types``):
    :class:`SeismicElements`, :class:`SeismicSecurityParams`,
    :class:`UnsignedSeismicTx`, :class:`TxSeismicMetadata`,
    :class:`Signature`, :class:`LegacyFields`,
    :class:`PlaintextTx`, :class:`DebugWriteResult`

**Client** (``seismic_web3.client``):
    :class:`EncryptionState`, :func:`get_encryption`,
    :func:`create_shielded_web3`, :func:`create_async_shielded_web3`

**Contract** (``seismic_web3.contract.shielded``):
    :class:`ShieldedContract`, :class:`AsyncShieldedContract`

**Module** (``seismic_web3.module``):
    :class:`SeismicNamespace`, :class:`AsyncSeismicNamespace`
"""

__version__ = "0.1.0"

# -- Types -------------------------------------------------------------------
from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
)

# -- Chains ------------------------------------------------------------------
from seismic_web3.chains import (
    SANVIL,
    SEISMIC_TESTNET,
    SEISMIC_TX_TYPE,
    ChainConfig,
    make_seismic_testnet,
)

# -- Client ------------------------------------------------------------------
from seismic_web3.client import (
    EncryptionState,
    create_async_shielded_web3,
    create_shielded_web3,
    get_encryption,
)

# -- Contract ----------------------------------------------------------------
from seismic_web3.contract.shielded import (
    AsyncShieldedContract,
    ShieldedContract,
)

# -- Module ------------------------------------------------------------------
from seismic_web3.module import (
    AsyncSeismicNamespace,
    SeismicNamespace,
)

# -- Transaction types -------------------------------------------------------
from seismic_web3.transaction_types import (
    DebugWriteResult,
    LegacyFields,
    PlaintextTx,
    SeismicElements,
    SeismicSecurityParams,
    Signature,
    TxSeismicMetadata,
    UnsignedSeismicTx,
)

__all__ = [
    "SANVIL",
    "SEISMIC_TESTNET",
    "SEISMIC_TX_TYPE",
    "AsyncSeismicNamespace",
    "AsyncShieldedContract",
    "Bytes32",
    "ChainConfig",
    "CompressedPublicKey",
    "DebugWriteResult",
    "EncryptionNonce",
    "EncryptionState",
    "LegacyFields",
    "PlaintextTx",
    "PrivateKey",
    "SeismicElements",
    "SeismicNamespace",
    "SeismicSecurityParams",
    "ShieldedContract",
    "Signature",
    "TxSeismicMetadata",
    "UnsignedSeismicTx",
    "create_async_shielded_web3",
    "create_shielded_web3",
    "get_encryption",
    "make_seismic_testnet",
]
