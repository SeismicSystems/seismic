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

**Lookup responses** (``seismic_web3.transaction_response``):
    :class:`SeismicTransactionResponse`, :data:`GasPaymentResponse`,
    :func:`is_seismic_transaction`

**Client factories** (``seismic_web3.client``):
    :class:`EncryptionState`, :func:`get_encryption`,
    :func:`create_wallet_client`, :func:`create_async_wallet_client`,
    :func:`create_public_client`, :func:`create_async_public_client`

**Contract** (``seismic_web3.contract``):
    :class:`ShieldedContract`, :class:`AsyncShieldedContract`,
    :class:`PublicContract`, :class:`AsyncPublicContract`,
    :func:`has_shielded_params`

**Module** (``seismic_web3.module``):
    :class:`SeismicNamespace`, :class:`AsyncSeismicNamespace`,
    :class:`SeismicPublicNamespace`, :class:`AsyncSeismicPublicNamespace`

**Explorer URL helpers** (``seismic_web3.explorer``):
    :func:`get_explorer_url`, :func:`tx_explorer_url`,
    :func:`address_explorer_url`, :func:`block_explorer_url`,
    :func:`token_explorer_url`

**EIP-712** (``seismic_web3.transaction.eip712``):
    :func:`sign_seismic_tx_eip712`, :func:`eip712_signing_hash`,
    :func:`domain_separator`, :func:`struct_hash`,
    :func:`build_seismic_typed_data`
"""

__version__ = "0.2.2"

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
from seismic_web3.contract.abi import has_shielded_params
from seismic_web3.contract.public import (
    AsyncPublicContract,
    PublicContract,
)
from seismic_web3.contract.shielded import (
    AsyncShieldedContract,
    ShieldedContract,
)
from seismic_web3.explorer import (
    AddressExplorerTab,
    BlockExplorerTab,
    ExplorerItem,
    ExplorerTab,
    TokenExplorerTab,
    TxExplorerTab,
    address_explorer_url,
    block_explorer_url,
    get_explorer_url,
    token_explorer_url,
    tx_explorer_url,
)
from seismic_web3.gas_payment import GasPayment

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

# -- Lookup responses --------------------------------------------------------
from seismic_web3.transaction_response import (
    GasPaymentResponse,
    SeismicTransactionResponse,
    is_seismic_transaction,
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
    "AddressExplorerTab",
    "AsyncPublicContract",
    "AsyncSeismicNamespace",
    "AsyncSeismicPublicNamespace",
    "AsyncShieldedContract",
    "BlockExplorerTab",
    "Bytes32",
    "ChainConfig",
    "CompressedPublicKey",
    "DebugWriteResult",
    "EncryptionNonce",
    "EncryptionState",
    "ExplorerItem",
    "ExplorerTab",
    "GasPayment",
    "GasPaymentResponse",
    "LegacyFields",
    "PlaintextTx",
    "PrivateKey",
    "PublicContract",
    "SeismicElements",
    "SeismicNamespace",
    "SeismicPublicNamespace",
    "SeismicSecurityParams",
    "SeismicTransactionResponse",
    "ShieldedContract",
    "Signature",
    "TokenExplorerTab",
    "TxExplorerTab",
    "TxSeismicMetadata",
    "UnsignedSeismicTx",
    "address_explorer_url",
    "block_explorer_url",
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
    "get_explorer_url",
    "has_shielded_params",
    "hex_to_bytes",
    "is_seismic_transaction",
    "make_seismic_testnet",
    "make_withdrawal_credentials",
    "sign_seismic_tx_eip712",
    "struct_hash",
    "token_explorer_url",
    "tx_explorer_url",
]
