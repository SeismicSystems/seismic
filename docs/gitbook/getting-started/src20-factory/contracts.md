---
description: Solidity interface reference for SRC20Factory and SRC20Token
icon: file-contract
---

# Contracts

## SRC20Factory

Pre-deployed on Seismic testnet at `0x87F850cbC2cFfac086F20d0d7307E12d06fA2127`.

```solidity
contract SRC20Factory {
    event TokenCreated(
        address indexed creator,
        address indexed token,
        string name,
        string symbol
    );

    address[] public tokens;

    function createToken(
        string memory name,
        string memory symbol,
        uint8 decimals,
        suint256 initialSupply
    ) external returns (address);

    function getTokenCount() external view returns (uint256);
}
```

### createToken

Deploys a new `SRC20Token`, records it in the `tokens` array, emits `TokenCreated`, and returns the deployed address. The caller becomes the token's `owner`.

`initialSupply` is a `suint256` — it is encrypted in the transaction and never visible on-chain. The supply is minted directly to `msg.sender` in the token's constructor.

### tokens

The public `tokens` array stores every token address in deployment order. Access by index:

```solidity
address token = factory.tokens(0);
```

### getTokenCount

Returns `tokens.length`. Use this to iterate the full list.

### TokenCreated event

Emitted on every successful deployment. The `name` and `symbol` fields are unencrypted strings; `creator` and `token` are indexed for log filtering.

---

## SRC20Token

Each token deployed by the factory is an instance of `SRC20Token`, which extends the base `SRC20` contract from `seismic-std-lib`.

```solidity
contract SRC20Token is SRC20 {
    address public owner;

    constructor(
        string memory _name,
        string memory _symbol,
        uint8 _decimals,
        suint256 _initialSupply,
        address _owner
    );

    function totalSupply() public view returns (uint256);
    function mint(address to, suint256 amount) external;   // owner only
    function burn(address from, suint256 amount) external; // owner only
}
```

### mint / burn

Only callable by `owner`. Both revert with `"NOT_OWNER"` if called by anyone else. The `amount` parameter is `suint256` — it is shielded in transit.

### Inherited from SRC20

| Function                                                                    | Description                                               |
| --------------------------------------------------------------------------- | --------------------------------------------------------- |
| `transfer(address to, suint256 amount) → bool`                              | Encrypted transfer to `to`                                |
| `transferFrom(address from, address to, suint256 amount) → bool`            | Encrypted transferFrom                                    |
| `approve(address spender, suint256 amount) → bool`                          | Set encrypted allowance                                   |
| `balance() → uint256`                                                       | Caller's own encrypted balance (requires a signed read)   |
| `allowance(address spender) → uint256`                                      | Caller's allowance for `spender` (requires a signed read) |
| `balanceOfSigned(address owner, uint256 expiry, bytes signature) → uint256` | Signed read pattern for balance                           |
| `permit(...)`                                                               | EIP-2612 permit with encrypted amounts                    |

`balance()` and `allowance()` use Seismic's signed-read mechanism — the caller must submit an off-chain signature that authorizes the node to decrypt and return their private state. See [Signed Reads](../../reference/seismic-transaction/signed-reads.md) for details.

### Why not clones?

The factory deploys a full `SRC20Token` contract for each token rather than using EIP-1167 minimal proxies. `SRC20` stores `decimals` as an immutable field and computes the EIP-2612 domain separator in the constructor — both must be baked into bytecode at deployment time. Clones share bytecode and cannot carry per-instance immutables, so they are not compatible with `SRC20`.
