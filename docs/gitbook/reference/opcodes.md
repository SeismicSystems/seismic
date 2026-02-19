---
icon: wave-sine
---

# Opcodes

We introduce two new EVM instructions to handle confidential storage:

#### 5.1 `CSTORE`

- **Purpose**: Stores shielded values in marked confidential storage slots.
- **Behavior**: Sets the storage slot as confidential during the store operation.

#### 5.2 `CLOAD`

- **Purpose**: Retrieves shielded values from marked confidential or uninitialized storage slots.
- **Behavior**: Only accesses storage slots marked as confidential.

#### 5.3 Storage Rights Management

- **Flagged Storage**: We introduce `FlaggedStorage` to tag storage slots as public or private based on the store instructions (`SSTORE` for public, `CSTORE` for confidential).
- **Access Control**:
  - **Public Storage**: Can be stored and loaded using `SSTORE` and `SLOAD`.
  - **Confidential Storage**: Must be stored using `CSTORE` and loaded using `CLOAD`.
- **Flexibility**: Storage slots are not permanently fixed as public or private. Developers can manage access rights using inline assembly if needed. Otherwise, the compiler will take care of it.
