// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "seismic-std-lib/SRC20.sol";
import {Initializable} from "@openzeppelin/contracts/proxy/utils/Initializable.sol";

/// @notice A basic USDC-like stablecoin built on SRC20 with shielded balances.
/// @dev 6 decimals, admin-controlled minting/burning, pausable.
///
/// Deployable two ways:
/// - Directly: the constructor sets metadata and `admin = msg.sender`.
/// - Behind an ERC-1967 proxy: pass `abi.encodeCall(SUSDC.initialize, (admin))`
///   as the proxy's `_data` so initialization is atomic with deployment. The
///   constructor calls `_disableInitializers()`, locking the implementation
///   itself against initialization.
///
/// Upgrade-safety: SRC20's storage layout is the base of this contract's
/// layout. Never reorder or insert variables in SRC20 or before `admin`;
/// future versions may only append new variables after `paused`.
contract SUSDC is SRC20, Initializable {
    address public admin;
    bool public paused;

    event AdminTransferred(address indexed oldAdmin, address indexed newAdmin);
    event Paused(address indexed account);
    event Unpaused(address indexed account);

    modifier onlyAdmin() {
        require(msg.sender == admin, "SUSDC: caller is not admin");
        _;
    }

    modifier whenNotPaused() {
        require(!paused, "SUSDC: paused");
        _;
    }

    constructor() SRC20("Shielded USD Coin", "SUSDC", 6) {
        admin = msg.sender;
        _disableInitializers();
    }

    /// @notice Initializer for proxy deployments, where the constructor only
    /// ran on the implementation and never touched the proxy's storage.
    /// `decimals` needs no initialization: it is an immutable baked into the
    /// implementation's bytecode, so it reads as 6 through the proxy.
    function initialize(address initialAdmin) external initializer {
        require(initialAdmin != address(0), "SUSDC: zero address");
        name = "Shielded USD Coin";
        symbol = "SUSDC";
        admin = initialAdmin;
    }

    /// @dev Always compute the domain separator instead of using the immutable
    /// cached at construction: behind a proxy, the cached value would carry the
    /// implementation's address as `verifyingContract`, breaking `permit()`
    /// signatures made against the proxy.
    function DOMAIN_SEPARATOR() public view override returns (bytes32) {
        return computeDomainSeparator();
    }

    function mint(address to, suint256 amount) external onlyAdmin {
        _mint(to, amount);
    }

    function burn(address from, suint256 amount) external onlyAdmin {
        _burn(from, amount);
    }

    function pause() external onlyAdmin {
        paused = true;
        emit Paused(msg.sender);
    }

    function unpause() external onlyAdmin {
        paused = false;
        emit Unpaused(msg.sender);
    }

    function transferAdmin(address newAdmin) external onlyAdmin {
        require(newAdmin != address(0), "SUSDC: zero address");
        emit AdminTransferred(admin, newAdmin);
        admin = newAdmin;
    }

    function transfer(address to, suint256 amount) public override whenNotPaused returns (bool) {
        return super.transfer(to, amount);
    }

    function transferFrom(address from, address to, suint256 amount) public override whenNotPaused returns (bool) {
        return super.transferFrom(from, to, amount);
    }

    function approve(address spender, suint256 amount) public override whenNotPaused returns (bool) {
        return super.approve(spender, amount);
    }

    function totalSupply() external view returns (uint256) {
        return _totalSupply();
    }

    /*//////////////////////////////////////////////////////////////
                         ERC20 COMPATIBILITY
    //////////////////////////////////////////////////////////////*/

    // Overloads with standard ERC20 types so the canonical selectors
    // (transfer 0xa9059cbb, transferFrom 0x23b872dd, approve 0x095ea7b3,
    // balanceOf 0x70a08231, allowance 0xdd62ed3e) resolve instead of hitting
    // the fallback when called by contracts written against IERC20.
    // The amount in these paths is public: it appears in plaintext calldata
    // or in the calling contract's logic, revealing a delta on the shielded
    // balances it touches.

    function transfer(address to, uint256 amount) public returns (bool) {
        return transfer(to, suint256(amount));
    }

    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        return transferFrom(from, to, suint256(amount));
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        return approve(spender, suint256(amount));
    }

    /// @notice ERC20-compatible balance query, restricted to the account itself
    /// so it cannot be used to read other users' shielded balances.
    function balanceOf(address account) public view returns (uint256) {
        require(msg.sender == account, "SUSDC: caller is not account");
        return uint256(balances[account]);
    }

    /// @notice ERC20-compatible allowance query, restricted to the owner and
    /// spender so it cannot be used to read other users' shielded allowances.
    function allowance(address owner, address spender) public view returns (uint256) {
        require(msg.sender == owner || msg.sender == spender, "SUSDC: caller is not owner or spender");
        return uint256(allowances[owner][spender]);
    }
}
