---
name: ssol-test-mirror
description: Mirror upstream Solidity tests as shielded type equivalents (suint, sint, saddress, sbool, sbytes) for seismic-solidity
---

# Mirror Upstream Tests as Shielded Equivalents

Given a **shielded type** (e.g., `suint`, `sint`, `saddress`, `sbool`, `sbytes`), find all upstream Solidity tests for the corresponding unshielded type and create shielded equivalents that don't already exist.

## Invocation

Usage: `/ssol-test-mirror <shielded-type>`

Example: `/ssol-test-mirror saddress`

The user specifies a **shielded type** — one of `suint`, `sint`, `sbool`, `saddress`, or `sbytes`. There is no default; the type must always be provided.

The skill then:

1. **Identifies the unshielded counterpart** (`suint` → `uint`, `sint` → `int`, `sbool` → `bool`, `saddress` → `address`, `sbytes` → `bytes`)

2. **Scans the test suite** for all upstream tests exercising that unshielded type across `test/libsolidity/semanticTests/`, `test/libsolidity/syntaxTests/`, and `test/cmdlineTests/`. Search broadly — relevant tests live in many subdirectories (arithmetics, operators/shifts, cleanup, expressions, exponentiation, array, storage, types, abiEncoderV2, etc.).

3. **Checks for existing shielded versions**. For each upstream test, check if a `shielded_` prefixed version already exists in the same directory.

4. **Reports the gap**. Present the user with a summary of:
   - Upstream test files that need shielded counterparts (no `shielded_` version exists)
   - Files that already have shielded versions (no action needed)
   - Files that should be skipped (with reasons per the skip criteria in Section 9)

5. **Adapts on approval**. After the user reviews and approves the list, create shielded versions using the adaptation rules below.

The user may also provide specific file paths or glob patterns to narrow the scope instead of scanning the full test suite.

## Post-Merge Mode

After an upstream merge from the `develop` branch, this skill can also identify **only the newly added** upstream tests that need shielded counterparts. Use this after merges to keep the shielded test suite in sync.

**Steps:**

1. **Find the merge commit(s)**. Look at recent history on the `seismic` branch for merge commits from `develop`:
   ```bash
   git log --merges --first-parent --oneline -10
   ```

2. **Identify new test files added by the merge**. Diff the merge commit against its first parent to find added test files:
   ```bash
   git diff --name-only --diff-filter=A <merge-commit>^1 <merge-commit> -- test/libsolidity/semanticTests/ test/libsolidity/syntaxTests/ test/cmdlineTests/
   ```

3. **Filter to the target type**. From the added files, identify tests that exercise the unshielded counterpart of the requested type. Read each file to confirm relevance. Skip tests that fall under the skip criteria (Section 9).

4. **Check for existing shielded versions**. For each candidate, check if a `shielded_` prefixed version already exists in the same directory.

5. **Report and adapt**. Same as the main flow — present the gap, then adapt on approval.

If no merge commit is specified, default to the most recent merge from `develop`. The user can also specify a commit range (e.g., `v0.8.28..HEAD`) to scan across multiple merges.

## Output File Naming

- **Semantic/syntax tests**: Prefix the filename with `shielded_` (e.g., `shift_left.sol` → `shielded_shift_left.sol`). Place the file in the same directory as the source.
- **Cmdline tests**: Prefix the directory name with `shielded_` (e.g., `suint8_via_ir/` → `shielded_suint8_via_ir/`). Each cmdline test directory contains `input.sol`, `args`, and `output`. The `output` file must be captured by compiling `input.sol` with the flags in `args` — use `--unsafe-via-ir` instead of `--via-ir`.
- If a `shielded_` version already exists, warn the user and skip unless they confirm overwrite

## Adaptation Rules

Apply ALL of the following transformations:

### 1. Type Substitution

Swap the base type for its shielded equivalent in parameters, locals, and storage variables:

| Source Type | Shielded Type |
|-------------|---------------|
| `uintN`     | `suintN`      |
| `intN`      | `sintN`       |
| `address`   | `saddress`    |
| `bool`      | `sbool`       |
| `bytesN`    | `sbytesN`     |

**Do NOT change:**
- Return types of `public`/`external` functions — these must stay unshielded
- Types used as mapping keys or array indices — shielded types are disallowed there
- Types used in `abi.encode`/`abi.decode` calls — shielded types cannot be ABI encoded
- Types used in events — shielded types cannot appear in event parameters
- Types used as `constant` or `immutable` — shielded types cannot be constant/immutable

### 2. Return Type Pattern

Public/external functions must return unshielded types. Cast the shielded result at the return statement:

```solidity
// BEFORE (upstream)
function f(uint16 a, uint16 b) public returns (uint16) {
    return a + b;
}

// AFTER (shielded)
function f(suint16 a, suint16 b) public returns (uint16) {
    return uint16(a + b);
}
```

For `internal`/`private` functions that are only called by other internal functions, shielded return types are fine.

### 3. Function Signatures in Test Expectations

Update the function signatures in `// ----` expectation comments to use shielded types:

```solidity
// BEFORE: // f(uint16,uint16): 65534, 0 -> 0xfffe
// AFTER:  // f(suint16,suint16): 65534, 0 -> 0xfffe
```

The input values and expected outputs stay the same — shielded types have the same arithmetic semantics.

### 4. Literal Wrapping

All literal values assigned to shielded variables must be explicitly wrapped:

```solidity
// BEFORE: uint x = 42;
// AFTER:  suint x = suint(42);

// BEFORE: uint8 y = 0xff;
// AFTER:  suint8 y = suint8(0xff);
```

This includes `type(uintN).max` → `type(suintN).max`.

### 5. Assembly: Storage Opcodes

For **shielded** storage variables, replace storage opcodes:

| Regular | Shielded |
|---------|----------|
| `sstore(var.slot, val)` | `cstore(var.slot, val)` |
| `sload(var.slot)` | `cload(var.slot)` |

**Keep `sstore`/`sload` unchanged** for regular (non-shielded) storage variables. Many tests intentionally mix both to compare behavior.

When a test uses assembly to write garbled/dirty values into storage for cleanup testing, use `cstore` for shielded vars and `sstore` for regular vars.

### 6. Dynamic Array Length Returns `suint256`

For **dynamic** shielded arrays (`suintN[]`, etc.), `.length` returns `suint256`. You must cast it before using in arithmetic, comparisons, or as an array index. Fixed-length shielded arrays (`suintN[K]`) return an unshielded length as normal.

```solidity
// BEFORE: while (data.length > 1) data.pop();
// AFTER:  while (uint256(data.length) > 1) data.pop();

// BEFORE: data[data.length - 1] = 42;
// AFTER:  data[uint256(data.length) - 1] = suint(42);
```

Also use `cstore` (not `sstore`) when writing to a shielded array's length slot in assembly:

```solidity
assembly { cstore(x.slot, 20) }  // set shielded array length
```

### 7. Shielded Type Restrictions

If the upstream test uses any of these patterns, they need special handling:

| Pattern | Problem | Solution |
|---------|---------|----------|
| `public` state variable | Auto-getter would return shielded type | Make `private` + add explicit getter returning unshielded |
| `constant` variable | Shielded types can't be constant | Use `private` storage variable instead |
| `immutable` variable | Shielded types can't be immutable | Use `private` storage variable set in constructor |
| `abi.encode(shieldedVar)` | Shielded types can't be ABI encoded | Cast to unshielded first: `abi.encode(uint(x))` |
| Event parameter | Shielded types can't be in events | Cast to unshielded: `emit E(uint(x))` |
| Mapping key | Shielded types can't be mapping keys | Keep key type unshielded |
| Array index | Shielded types can't be array indices | Cast index: `arr[uint(i)]` |
| `addmod`/`mulmod` builtins | Expect `uint256` args | Cast: `addmod(uint256(a), uint256(b), uint256(c))` |

### 8. Signed Integer Tests

When adapting `int` tests to `sint`:
- Only adapt the unsigned portions if the test mixes signed/unsigned and the signed portion has different overflow semantics
- `sint` has the same two's complement behavior as `int`

### 9. Skip Criteria

**Do not adapt** these types of tests:
- Pure Yul/assembly tests with no Solidity-level type usage
- Tests that are fundamentally about ABI encoding v1 behavior
- Tests that rely on storage packing arithmetic (shielded types don't pack)
- Tests where the entire point is `public` getter behavior (shielded types can't have public getters — adapt only if a private+getter pattern makes sense)

When skipping, tell the user why.

## Complete Example: Arithmetic Test

**Source** (`checked_add_v2.sol`):
```solidity
contract C {
    function f(uint16 a, uint16 b) public returns (uint16) {
        return a + b;
    }
}
// ----
// f(uint16,uint16): 65534, 0 -> 0xfffe
// f(uint16,uint16): 65536, 0 -> FAILURE
// f(uint16,uint16): 65535, 0 -> 0xffff
// f(uint16,uint16): 65535, 1 -> FAILURE, hex"4e487b71", 0x11
```

**Adapted** (`shielded_checked_add_v2.sol`):
```solidity
contract C {
    function f(suint16 a, suint16 b) public returns (uint16) {
        return uint16(a + b);
    }
}
// ----
// f(suint16,suint16): 65534, 0 -> 0xfffe
// f(suint16,suint16): 65536, 0 -> FAILURE
// f(suint16,suint16): 65535, 0 -> 0xffff
// f(suint16,suint16): 65535, 1 -> FAILURE, hex"4e487b71", 0x11
```

## Complete Example: Storage + Assembly Test

**Source** (`unchecked_cleanup.sol`):
```solidity
contract C {
    uint8 a;
    uint8 b;
    function testDiv() public returns (uint) {
        assembly {
            sstore(a.slot, 0x0102)
            sstore(b.slot, 0x0101)
        }
        unchecked { return a / b; }
    }
}
```

**Adapted** (`shielded_unchecked_cleanup.sol`):
```solidity
contract C {
    suint8 a;
    suint8 b;

    function testDiv() public returns (uint) {
        assembly {
            cstore(a.slot, 0x0102)
            cstore(b.slot, 0x0101)
        }
        unchecked {
            return uint(a / b);
        }
    }
}
// ----
// testDiv() -> 2
```

## Complete Example: Array Test

**Source** (`array_copy_cleanup_uint40.sol`):
```solidity
contract C {
    uint40[] x;
    function f() public returns(bool) {
        for (uint i = 0; i < 20; i++) x.push(42);
        while (x.length > 1) x.pop();
        x[0] = 23;
        assembly { sstore(x.slot, 20) }
        // ... assertions ...
    }
}
```

**Adapted** (`shielded_array_copy_cleanup_uint40.sol`):
```solidity
contract C {
    suint40[] x;
    function f() public returns(bool) {
        for (uint i = 0; i < 20; i++) x.push(suint40(42));
        while (uint256(x.length) > 1) x.pop();
        x[0] = suint40(23);
        assembly { cstore(x.slot, 20) }
        assert(uint40(x[0]) == 23);
        assert(uint40(x[1]) == 0);
        // ... etc ...
    }
}
```

## After Writing Tests

Use the `/ssolc-tests` skill for detailed build and test commands. Quick summary:

1. **Build the compiler** (if not already built):
   ```bash
   mkdir -p build && cd build && cmake .. && make -j$(nproc)
   ```

2. **Syntax tests**: Verify with isoltest (always pass `--no-semantic-tests`):
   ```bash
   build/test/tools/isoltest --no-semantic-tests -t "<test-filter>"
   ```
   Use `--accept-updates` only with explicit user approval to fix byte offsets in error expectations.

3. **Semantic tests**: Run via seismic-revm's `revme` binary (see `/ssolc-tests` for full commands with all optimizer/via-ir configurations):
   ```bash
   cd <seismic-revm-repo-root> && cargo run -p revme -- semantics \
     --keep-going --unsafe-via-ir \
     -s "<solidity-repo-root>/build/solc/solc" \
     -t "<solidity-repo-root>/test/libsolidity/semanticTests"
   ```

4. If a test fails, diagnose whether it's an adaptation error or a compiler bug. Fix adaptation errors; report compiler bugs to the user.
