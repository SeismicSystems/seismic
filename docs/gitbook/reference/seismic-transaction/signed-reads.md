---
description: >-
  Seismic introduces the signed read, so that contracts can validate msg.sender
  in view functions. Seismic nodes will zero out the "from" address of an
  unsigned vanilla eth_call so it cannot be spoofed
---

# Signed reads

* The Seismic transaction has a counterpart, which we call “signed reads”
* The motivation for a signed read is: in the EVM, users can make an `eth_call` and set the “from” field to any address they’d like to spoof
* We want to give contract developers the ability to validate contract reads against msg.sender. For example, a user could implement an ERC20 token where only the owner can view their balance
* To solve this, we do two things: (1) we zero out the “from” field when users make a vanilla `eth_call` and (2) we created the “signed read” to allow users to make a call with a specific `from` address
* Signed reads are sent to Seismic's `eth_call` endpoint in the same way we would send a signed Seismic transaction to `eth_sendRawTransaction`. This can be either with a normal raw transaction, or an EIP-712 transaction
* Signed reads must be a valid Seismic transaction (type `0x4a`). They cannot be made with any other transaction type
* The response to a signed read will be encrypted to the encryption public key included in the transaction's Seismic elements. Therefore anyone who manages to intercept the response still cannot decrypt the response to the signed read.
* Users can set the `signed_read` field in SeismicElements to `true`. We encourage this, and it is the default behaviour in our client implementations. The purpose of this is to prevent anyone who has managed to intercept this `eth_call` from sending that same payload to `eth_sendRawTransaction`, which would otherwise trigger a write transaction. When validating a transaction before it hits the pool, we check if the `signed_read` field is set to true; if it is, the transaction is rejected
  * This does not apply the other way around. Meaning, if `signed_read` is `false` (and the user wants to create a transaction), it can still be passed to `eth_call`. We think this validation is unnecessary because:
    * For an attacked to decrypt the response to a signed read, they’d need the user’s secret encryption key
    * The account's transaction nonce will increment right after their Seismic transaction is included in a block, after which the read will fail
    * This also allows `eth_estimateGas` to function properly; otherwise it would not pass validation on the `signed_read` field
