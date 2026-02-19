"""SRC20 event watching with viewing keys.

This subpackage provides tools for watching SRC20 Transfer and
Approval events on-chain and decrypting the encrypted amounts
using AES-256 viewing keys stored in the Directory genesis contract.

Quick start
-----------

**Watch your own events (wallet client):**

.. code-block:: python

    from seismic_web3.src20 import watch_src20_events

    watcher = watch_src20_events(
        w3,
        encryption=w3.seismic.encryption,
        private_key=pk,
        token_address="0x...",
        on_transfer=lambda log: print(log.decrypted_amount),
    )
    # later…
    watcher.stop()

**Watch with an explicit viewing key:**

.. code-block:: python

    from seismic_web3.src20 import watch_src20_events_with_key

    watcher = watch_src20_events_with_key(
        w3,
        viewing_key=Bytes32(key_hex),
        token_address="0x...",
        on_transfer=lambda log: print(log.decrypted_amount),
    )

**Directory helpers:**

.. code-block:: python

    from seismic_web3.src20 import register_viewing_key, get_viewing_key
"""

# -- Types ---------------------------------------------------------------
from seismic_web3.src20.types import (
    DecryptedApprovalLog,
    DecryptedTransferLog,
)

# -- Crypto --------------------------------------------------------------
from seismic_web3.src20.crypto import (
    decrypt_encrypted_amount,
    parse_encrypted_data,
)

# -- Directory helpers ---------------------------------------------------
from seismic_web3.src20.directory import (
    async_check_has_key,
    async_get_key_hash,
    async_get_viewing_key,
    async_register_viewing_key,
    check_has_key,
    compute_key_hash,
    get_key_hash,
    get_viewing_key,
    register_viewing_key,
)

# -- Watchers ------------------------------------------------------------
from seismic_web3.src20.watch import (
    AsyncSRC20EventWatcher,
    SRC20EventWatcher,
    async_watch_src20_events,
    async_watch_src20_events_with_key,
    watch_src20_events,
    watch_src20_events_with_key,
)

__all__ = [
    # Types
    "DecryptedApprovalLog",
    "DecryptedTransferLog",
    # Crypto
    "decrypt_encrypted_amount",
    "parse_encrypted_data",
    # Directory
    "async_check_has_key",
    "async_get_key_hash",
    "async_get_viewing_key",
    "async_register_viewing_key",
    "check_has_key",
    "compute_key_hash",
    "get_key_hash",
    "get_viewing_key",
    "register_viewing_key",
    # Watchers
    "AsyncSRC20EventWatcher",
    "SRC20EventWatcher",
    "async_watch_src20_events",
    "async_watch_src20_events_with_key",
    "watch_src20_events",
    "watch_src20_events_with_key",
]
