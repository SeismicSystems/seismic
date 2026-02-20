"""Protocol-level constants shared across the SDK.

These live in their own module to avoid circular imports between
``chains`` and the transaction/client modules.
"""

#: Seismic custom transaction type byte (decimal 74, hex 0x4a).
SEISMIC_TX_TYPE: int = 0x4A

#: Chain ID for the Seismic public testnet.
SEISMIC_TESTNET_CHAIN_ID: int = 5124

#: Chain ID for the local Sanvil (Seismic Anvil) instance.
SANVIL_CHAIN_ID: int = 31_337

#: EIP-712 typed-data message version for JSON-RPC wallet signing.
TYPED_DATA_MESSAGE_VERSION: int = 2
