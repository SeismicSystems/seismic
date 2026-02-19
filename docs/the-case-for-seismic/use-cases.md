---
icon: lightbulb
---

# Use Cases

Seismic's shielded types let you add privacy to any smart contract pattern. Below are five categories with code snippets showing how each looks in Seismic Solidity.

## Private tokens (SRC20)

An ERC20 with shielded balances and transfer amounts. Users can check their own balance through signed reads, but no one else can see it.

```solidity
mapping(address => suint256) balanceOf;
mapping(address => mapping(address => suint256)) allowance;
suint256 totalSupply;

function transfer(address to, suint256 amount) public {
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```

The only difference from a standard ERC20: `uint256` becomes `suint256`. The compiler routes all balance reads and writes through shielded storage automatically. Observers see `0x000` for all balances and amounts.

For a full walkthrough, see the [SRC20 tutorial](../tutorials/src20/README.md).

## Confidential DeFi

An AMM where liquidity positions, reserves, and swap amounts are hidden from observers. This prevents front-running and MEV extraction because bots cannot see the pool state or pending swaps.

```solidity
suint256 reserve0;
suint256 reserve1;
mapping(address => suint256) liquidity;

function swap(address tokenIn, suint256 amountIn) public returns (suint256 amountOut) {
    if (tokenIn == token0) {
        amountOut = (amountIn * reserve1) / (reserve0 + amountIn);
        reserve0 += amountIn;
        reserve1 -= amountOut;
    } else {
        amountOut = (amountIn * reserve0) / (reserve1 + amountIn);
        reserve1 += amountIn;
        reserve0 -= amountOut;
    }
}
```

The constant-product formula is standard AMM logic. Shielding the reserves and amounts means no one can compute the price impact of a pending swap or sandwich a user's trade.

## Compliant finance (Intelligence Contracts)

Privacy with built-in access control. A compliance officer can view balances for regulatory purposes, but the general public cannot. This pattern is sometimes called an "Intelligence Contract" -- a private contract that selectively reveals information to authorized parties.

```solidity
import "@openzeppelin/contracts/access/AccessControl.sol";

bytes32 public constant COMPLIANCE_ROLE = keccak256("COMPLIANCE_ROLE");

mapping(address => suint256) balanceOf;

function getBalance(address account) public view returns (uint256) {
    require(
        msg.sender == account || hasRole(COMPLIANCE_ROLE, msg.sender),
        "Not authorized"
    );
    return uint256(balanceOf[account]);
}

function transfer(address to, suint256 amount) public {
    balanceOf[msg.sender] -= amount;
    balanceOf[to] += amount;
}
```

Balances are stored as `suint256` so they are shielded by default. The `getBalance` function casts the shielded value to a regular `uint256` for return, but only if the caller is the account owner or holds the `COMPLIANCE_ROLE`. Everyone else is rejected.

## Private voting

Secret ballot governance on-chain. Votes are hidden during the voting period. The tally can be revealed when voting ends, but individual votes remain private.

```solidity
mapping(address => sbool) hasVoted;
suint256 yesVotes;
suint256 noVotes;
uint256 public votingEnd;

function vote(sbool inFavor) public {
    require(block.timestamp < votingEnd, "Voting ended");
    require(!bool(hasVoted[msg.sender]), "Already voted");

    hasVoted[msg.sender] = sbool(true);
    if (bool(inFavor)) {
        yesVotes += suint256(1);
    } else {
        noVotes += suint256(1);
    }
}

function getResults() public view returns (uint256 yes, uint256 no) {
    require(block.timestamp >= votingEnd, "Voting still open");
    yes = uint256(yesVotes);
    no = uint256(noVotes);
}
```

During voting, both the individual votes (`hasVoted`) and the running tallies (`yesVotes`, `noVotes`) are shielded. No one can see which way the vote is trending. After the deadline, `getResults()` casts the shielded tallies to public `uint256` values so the outcome can be read.

## Sealed-bid auctions

Bids are hidden until the auction closes. No bidder can see what others have bid, eliminating bid sniping and strategic underbidding.

```solidity
mapping(address => suint256) bids;
suint256 highestBid;
saddress highestBidder;
uint256 public auctionEnd;

function bid(suint256 amount) public {
    require(block.timestamp < auctionEnd, "Auction ended");
    bids[msg.sender] = amount;

    if (uint256(amount) > uint256(highestBid)) {
        highestBid = amount;
        highestBidder = saddress(msg.sender);
    }
}

function getWinner() public view returns (address winner, uint256 amount) {
    require(block.timestamp >= auctionEnd, "Auction still open");
    winner = address(highestBidder);
    amount = uint256(highestBid);
}
```

Each bid is stored as a `suint256`, and the highest bidder's address is stored as an `saddress`. During the auction, the contract tracks the leading bid internally, but no one outside the TEE can see any bid amounts or the current leader. After the auction ends, `getWinner()` reveals the result by casting shielded values to their public counterparts.
