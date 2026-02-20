"""Seismic Python SDK -- web3.py extensions for the Seismic privacy-enabled EVM.

Public API
----------

**Types** (``seismic_web3._types``):
    :class:`Bytes32`, :class:`PrivateKey`, :class:`CompressedPublicKey`,
    :class:`EncryptionNonce`, :func:`hex_to_bytes`

**Chains** (``seismic_web3.chains``):
    :class:`ChainConfig`, :data:`SEISMIC_TESTNET`, :data:`SANVIL`,
    :func:`make_seismic_testnet`, :data:`SEISMIC_TX_TYPE`

**Transaction types** (``seismic_web3.transaction_types``):
    :class:`SeismicElements`, :class:`SeismicSecurityParams`,
    :class:`UnsignedSeismicTx`, :class:`TxSeismicMetadata`,
    :class:`Signature`, :class:`LegacyFields`,
    :class:`PlaintextTx`, :class:`DebugWriteResult`

**Client factories** (``seismic_web3.client``):
    :class:`EncryptionState`, :func:`get_encryption`,
    :func:`create_wallet_client`, :func:`create_async_wallet_client`,
    :func:`create_public_client`, :func:`create_async_public_client`

**Contract** (``seismic_web3.contract``):
    :class:`ShieldedContract`, :class:`AsyncShieldedContract`,
    :class:`PublicContract`, :class:`AsyncPublicContract`

**Module** (``seismic_web3.module``):
    :class:`SeismicNamespace`, :class:`AsyncSeismicNamespace`,
    :class:`SeismicPublicNamespace`, :class:`AsyncSeismicPublicNamespace`

**EIP-712** (``seismic_web3.transaction.eip712``):
    :func:`sign_seismic_tx_eip712`, :func:`eip712_signing_hash`,
    :func:`domain_separator`, :func:`struct_hash`,
    :func:`build_seismic_typed_data`
"""

__version__ = "0.1.2"

# -- Types -------------------------------------------------------------------
from seismic_web3._types import (
    Bytes32,
    CompressedPublicKey,
    EncryptionNonce,
    PrivateKey,
    hex_to_bytes,
)

# -- ABIs --------------------------------------------------------------------
from seismic_web3.abis import (
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_ADDRESS,
    DIRECTORY_ABI,
    DIRECTORY_ADDRESS,
    SRC20_ABI,
    compute_deposit_data_root,
    make_withdrawal_credentials,
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
    create_async_public_client,
    create_async_shielded_web3,
    create_async_wallet_client,
    create_public_client,
    create_shielded_web3,
    create_wallet_client,
    get_encryption,
)

# -- Contract ----------------------------------------------------------------
from seismic_web3.contract.public import (
    AsyncPublicContract,
    PublicContract,
)
from seismic_web3.contract.shielded import (
    AsyncShieldedContract,
    ShieldedContract,
)

# -- Module ------------------------------------------------------------------
from seismic_web3.module import (
    AsyncSeismicNamespace,
    AsyncSeismicPublicNamespace,
    SeismicNamespace,
    SeismicPublicNamespace,
)

# -- EIP-712 ----------------------------------------------------------------
from seismic_web3.transaction.eip712 import (
    build_seismic_typed_data,
    domain_separator,
    eip712_signing_hash,
    sign_seismic_tx_eip712,
    struct_hash,
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
    "DEPOSIT_CONTRACT_ABI",
    "DEPOSIT_CONTRACT_ADDRESS",
    "DIRECTORY_ABI",
    "DIRECTORY_ADDRESS",
    "SANVIL",
    "SEISMIC_TESTNET",
    "SEISMIC_TX_TYPE",
    "SRC20_ABI",
    "AsyncPublicContract",
    "AsyncSeismicNamespace",
    "AsyncSeismicPublicNamespace",
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
    "PublicContract",
    "SeismicElements",
    "SeismicNamespace",
    "SeismicPublicNamespace",
    "SeismicSecurityParams",
    "ShieldedContract",
    "Signature",
    "TxSeismicMetadata",
    "UnsignedSeismicTx",
    "build_seismic_typed_data",
    "compute_deposit_data_root",
    "create_async_public_client",
    "create_async_shielded_web3",
    "create_async_wallet_client",
    "create_public_client",
    "create_shielded_web3",
    "create_wallet_client",
    "domain_separator",
    "eip712_signing_hash",
    "get_encryption",
    "hex_to_bytes",
    "make_seismic_testnet",
    "make_withdrawal_credentials",
    "sign_seismic_tx_eip712",
    "struct_hash",
]
