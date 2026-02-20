---
description: Emit transfer events with encrypted amounts using AES precompiles
icon: lock
---

# Encrypted Events

In the previous chapter, we cast `suint256` amounts to `uint256` before emitting events. This works, but it reveals the amount in the public event log. This chapter shows how to encrypt event data so that only the intended recipients can read it. _Estimated time: \~20 minutes._

## The problem

Events in Ethereum (and Seismic) are stored in transaction logs, which are public. You cannot use shielded types directly in event parameters:

```solidity
// This will NOT compile
event Transfer(address indexed from, address indexed to, suint256 amount);
```

The compiler rejects this because event data is written to public logs, and shielded types are only meaningful in contract storage. If you cast to `uint256` and emit, the amount appears in plaintext in the log -- defeating the purpose of shielding it in the first place.

## The solution

Use Seismic's AES-GCM precompiles to encrypt the sensitive data before emitting it. The event carries opaque bytes that only the intended recipient can decrypt.

The modified event signature uses `bytes` instead of `uint256` for the amount:

```solidity
event Transfer(address indexed from, address indexed to, bytes encryptedAmount);
```

The `from` and `to` addresses remain as `indexed` parameters. These are public -- observers can see who is transacting with whom. Only the amount is encrypted. If you need to hide the participants as well, you can encrypt those too, but that is less common for a token.

## Step by step

The encryption flow uses three of Seismic's precompiles:

### Step 1: Derive a shared secret with ECDH

The ECDH precompile at address `0x65` performs Elliptic Curve Diffie-Hellman key agreement. Given a private key and a public key, it produces a shared secret that both parties can independently derive.

For event encryption, the contract needs a keypair. You can generate one at deployment time and store the public key on-chain:

```solidity
bytes32 private contractPrivateKey;
bytes public contractPublicKey;

constructor(
    string memory _name,
    string memory _symbol,
    uint256 _initialSupply,
    bytes32 _contractPrivateKey,
    bytes memory _contractPublicKey
) {
    name = _name;
    symbol = _symbol;
    totalSupply = _initialSupply;
    balanceOf[msg.sender] = suint256(_initialSupply);
    contractPrivateKey = _contractPrivateKey;
    contractPublicKey = _contractPublicKey;
}
```

To derive a shared secret with a specific recipient, the contract calls the ECDH precompile with its own private key and the recipient's public key:

```solidity
function _deriveSharedSecret(bytes memory recipientPublicKey) internal view returns (bytes32) {
    // Call ECDH precompile at 0x65
    (bool success, bytes memory result) = address(0x65).staticcall(
        abi.encode(contractPrivateKey, recipientPublicKey)
    );
    require(success, "ECDH failed");
    return abi.decode(result, (bytes32));
}
```

### Step 2: Derive an encryption key with HKDF

The raw ECDH shared secret should not be used directly as an encryption key. The HKDF precompile at address `0x68` derives a proper cryptographic key from the shared secret:

```solidity
function _deriveEncryptionKey(bytes32 sharedSecret) internal view returns (bytes32) {
    // Call HKDF precompile at 0x68
    (bool success, bytes memory result) = address(0x68).staticcall(
        abi.encode(sharedSecret, "src20-transfer-event")
    );
    require(success, "HKDF failed");
    return abi.decode(result, (bytes32));
}
```

The second argument is a context string (sometimes called "info" in HKDF terminology). Using a unique context string for each purpose ensures that the same shared secret produces different keys for different uses.

### Step 3: Encrypt with AES-GCM

The AES-GCM Encrypt precompile at address `0x66` encrypts the data:

```solidity
function _encrypt(bytes32 key, bytes memory plaintext) internal view returns (bytes memory) {
    // Call AES-GCM Encrypt precompile at 0x66
    (bool success, bytes memory ciphertext) = address(0x66).staticcall(
        abi.encode(key, plaintext)
    );
    require(success, "Encryption failed");
    return ciphertext;
}
```

### Step 4: Emit the encrypted event

Putting it all together in an internal helper:

```solidity
function _emitEncryptedTransfer(
    address from,
    address to,
    suint256 amount,
    bytes memory recipientPublicKey
) internal {
    // Derive shared secret between contract and recipient
    bytes32 sharedSecret = _deriveSharedSecret(recipientPublicKey);

    // Derive encryption key
    bytes32 encKey = _deriveEncryptionKey(sharedSecret);

    // Encrypt the amount
    bytes memory plaintext = abi.encode(uint256(amount));
    bytes memory encryptedAmount = _encrypt(encKey, plaintext);

    // Emit with encrypted data
    emit Transfer(from, to, encryptedAmount);
}
```

## Full implementation

Here is the updated `transfer` function using encrypted events:

```solidity
// Mapping of address to their public key (registered on-chain)
mapping(address => bytes) public publicKeys;

function registerPublicKey(bytes memory pubKey) external {
    publicKeys[msg.sender] = pubKey;
}

function transfer(address to, suint256 amount) public returns (bool) {
    require(balanceOf[msg.sender] >= amount, "Insufficient balance");
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;

    // Encrypt amount for the recipient
    bytes memory recipientPubKey = publicKeys[to];
    if (recipientPubKey.length > 0) {
        _emitEncryptedTransfer(msg.sender, to, amount, recipientPubKey);
    } else {
        // Fallback: emit with zero if recipient has no registered key
        emit Transfer(msg.sender, to, bytes(""));
    }

    return true;
}
```

Users register their public key by calling `registerPublicKey` once. After that, any transfer they receive will emit an event encrypted to their key.

## Decrypting off-chain

The recipient can decrypt the event data by performing the reverse of the encryption flow:

1. Take the contract's public key (stored on-chain and readable by anyone).
2. Combine it with their own private key using ECDH to derive the same shared secret.
3. Run HKDF with the same context string (`"src20-transfer-event"`) to derive the same encryption key.
4. Decrypt the `encryptedAmount` from the event log using AES-GCM Decrypt.

In TypeScript with `seismic-viem`:

```typescript
import { decryptEventData } from "seismic-viem";

// Listen for Transfer events
const logs = await publicClient.getLogs({
  address: SRC20_ADDRESS,
  event: parseAbiItem(
    "event Transfer(address indexed from, address indexed to, bytes encryptedAmount)",
  ),
  fromBlock: "latest",
});

for (const log of logs) {
  // Decrypt the amount using the recipient's private key and the contract's public key
  const decryptedAmount = await decryptEventData({
    encryptedData: log.args.encryptedAmount,
    privateKey: recipientPrivateKey,
    contractPublicKey: contractPubKey,
    context: "src20-transfer-event",
  });

  console.log(`Transfer from ${log.args.from}: ${decryptedAmount} tokens`);
}
```

## Who can read what

Here is the visibility breakdown for each piece of data in a Transfer event:

| Data                | Who can see it | Why                                            |
| ------------------- | -------------- | ---------------------------------------------- |
| `from` address      | Everyone       | Indexed parameter, stored in public log topics |
| `to` address        | Everyone       | Indexed parameter, stored in public log topics |
| Transfer amount     | Recipient only | Encrypted to recipient's public key            |
| A transfer happened | Everyone       | The event emission itself is visible           |

The sender can also decrypt the amount because they know the plaintext -- they created the transaction. If you need the sender to be able to decrypt from the event log as well (for example, for transaction history), you can emit a second event encrypted to the sender's key, or encrypt to both keys and include both ciphertexts.

{% hint style="info" %}
Encrypted events add gas cost for the precompile calls. For applications where the event amount being public is acceptable, the simpler `uint256(amount)` cast from the previous chapter is more gas-efficient. Choose the approach that matches your privacy requirements.
{% endhint %}
