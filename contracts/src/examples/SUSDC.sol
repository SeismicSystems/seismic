// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "seismic-std-lib/SRC20.sol";
import {Initializable} from "@openzeppelin/contracts/proxy/utils/Initializable.sol";

/// @notice A basic USDC-like stablecoin built on SRC20 with shielded balances.
/// @dev 6 decimals, admin-controlled minting/burning, pausable. An optional
/// `bridge` (e.g. a Hyperlane `HypXERC20` router) may also mint and burn via the
/// plain-`uint256` overloads, which resolve to the xERC20/ERC20 selectors
/// (`mint(address,uint256)` 0x40c10f19, `burn(address,uint256)` 0x9dc29fac)
/// that bridge contracts written against IERC20-style interfaces invoke.
///
/// Deployable two ways:
/// - Directly: the constructor sets metadata and `admin = msg.sender`.
/// - Behind an ERC-1967 proxy: pass `abi.encodeCall(SUSDC.initialize, (admin))`
///   as the proxy's `_data` so initialization is atomic with deployment. The
///   constructor calls `_disableInitializers()`, locking the implementation
///   itself against initialization.
///
/// The bridge is rate limited per ERC-7281 (xERC20): `setLimits` gives it a
/// daily minting and burning allowance that replenishes linearly over 24h.
/// Limits default to zero, so a freshly set bridge cannot mint or burn until
/// the admin calls `setLimits`. The admin itself is never limited.
///
/// Upgrade-safety: SRC20's storage layout is the base of this contract's
/// layout. Never reorder or insert variables in SRC20 or before `admin`;
/// future versions may only append new variables after `bridgeLimits`.
contract SUSDC is SRC20, Initializable {
    address public admin;
    bool public paused;
    /// @notice Address allowed to mint and burn besides `admin` (zero = none).
    address public bridge;

    /// @dev ERC-7281 limit state for one direction (minting or burning).
    struct BridgeParameters {
        uint256 timestamp;
        uint256 ratePerSecond;
        uint256 maxLimit;
        uint256 currentLimit;
    }

    struct BridgeLimits {
        BridgeParameters minter;
        BridgeParameters burner;
    }

    /// @dev Per-bridge limits, keyed by bridge address like ERC-7281 so the
    /// standard getters work for any address. Only `bridge` can mint/burn.
    /// Appended after `bridge` for upgrade safety.
    mapping(address => BridgeLimits) internal bridgeLimits;

    /// @dev Window over which a bridge limit fully replenishes.
    uint256 private constant _DURATION = 1 days;

    event AdminTransferred(address indexed oldAdmin, address indexed newAdmin);
    event BridgeUpdated(address indexed oldBridge, address indexed newBridge);
    /// @dev ERC-7281 event; argument order follows the reference implementation.
    event BridgeLimitsSet(uint256 mintingLimit, uint256 burningLimit, address indexed bridge);

    /// @dev ERC-7281 errors.
    error IXERC20_NotHighEnoughLimits();
    error IXERC20_LimitsTooHigh();

    event Paused(address indexed account);
    event Unpaused(address indexed account);

    modifier onlyAdmin() {
        require(msg.sender == admin, "SUSDC: caller is not admin");
        _;
    }

    modifier onlyAdminOrBridge() {
        require(msg.sender == admin || msg.sender == bridge, "SUSDC: caller is not admin or bridge");
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

    function mint(address to, suint256 amount) external onlyAdminOrBridge {
        _useMintingLimit(uint256(amount));
        _mint(to, amount);
    }

    function burn(address from, suint256 amount) external onlyAdminOrBridge {
        _useBurningLimit(uint256(amount));
        _burn(from, amount);
    }

    /// @notice Set (or clear with zero) the bridge allowed to mint and burn.
    /// @dev The bridge's limits are kept per address; a new bridge starts at
    /// zero until `setLimits` is called for it.
    function setBridge(address newBridge) external onlyAdmin {
        emit BridgeUpdated(bridge, newBridge);
        bridge = newBridge;
    }

    /*//////////////////////////////////////////////////////////////
                      xERC20 (ERC-7281) BRIDGE LIMITS
    //////////////////////////////////////////////////////////////*/

    /// @notice ERC-7281 `owner()`: the admin controls bridge limits.
    function owner() external view returns (address) {
        return admin;
    }

    /// @notice Set the daily minting and burning allowance of a bridge.
    /// @dev Follows the ERC-7281 reference: the current allowance is adjusted
    /// by the change in the max limit, and replenishes at `max / 1 day`.
    function setLimits(address bridgeAddr, uint256 mintingLimit, uint256 burningLimit) external onlyAdmin {
        if (mintingLimit > type(uint256).max / 2 || burningLimit > type(uint256).max / 2) {
            revert IXERC20_LimitsTooHigh();
        }
        _changeLimit(bridgeLimits[bridgeAddr].minter, mintingLimit);
        _changeLimit(bridgeLimits[bridgeAddr].burner, burningLimit);
        emit BridgeLimitsSet(mintingLimit, burningLimit, bridgeAddr);
    }

    function mintingMaxLimitOf(address bridgeAddr) external view returns (uint256) {
        return bridgeLimits[bridgeAddr].minter.maxLimit;
    }

    function burningMaxLimitOf(address bridgeAddr) external view returns (uint256) {
        return bridgeLimits[bridgeAddr].burner.maxLimit;
    }

    function mintingCurrentLimitOf(address bridgeAddr) public view returns (uint256) {
        return _currentLimit(bridgeLimits[bridgeAddr].minter);
    }

    function burningCurrentLimitOf(address bridgeAddr) public view returns (uint256) {
        return _currentLimit(bridgeLimits[bridgeAddr].burner);
    }

    /// @dev Charges `amount` against the caller's minting allowance unless the
    /// caller is the admin, who is never limited.
    function _useMintingLimit(uint256 amount) internal {
        if (msg.sender == admin) return;
        _useLimit(bridgeLimits[msg.sender].minter, amount);
    }

    function _useBurningLimit(uint256 amount) internal {
        if (msg.sender == admin) return;
        _useLimit(bridgeLimits[msg.sender].burner, amount);
    }

    function _useLimit(BridgeParameters storage params, uint256 amount) private {
        uint256 current = _currentLimit(params);
        if (amount > current) revert IXERC20_NotHighEnoughLimits();
        params.currentLimit = current - amount;
        params.timestamp = block.timestamp;
    }

    function _changeLimit(BridgeParameters storage params, uint256 newMax) private {
        uint256 oldMax = params.maxLimit;
        uint256 current = _currentLimit(params);
        params.maxLimit = newMax;
        if (oldMax > newMax) {
            uint256 diff = oldMax - newMax;
            params.currentLimit = current > diff ? current - diff : 0;
        } else {
            params.currentLimit = current + (newMax - oldMax);
        }
        params.ratePerSecond = newMax / _DURATION;
        params.timestamp = block.timestamp;
    }

    function _currentLimit(BridgeParameters storage params) private view returns (uint256) {
        uint256 maxLimit = params.maxLimit;
        uint256 current = params.currentLimit;
        if (current == maxLimit) return current;
        if (params.timestamp + _DURATION <= block.timestamp) return maxLimit;
        uint256 replenished = current + (block.timestamp - params.timestamp) * params.ratePerSecond;
        return replenished > maxLimit ? maxLimit : replenished;
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

    /// @notice xERC20/ERC20-style mint used by bridge routers. Amount is public calldata.
    /// Charged against the bridge's minting limit.
    function mint(address to, uint256 amount) external onlyAdminOrBridge {
        _useMintingLimit(amount);
        _mint(to, suint256(amount));
    }

    /// @notice xERC20/ERC20-style burn used by bridge routers. Amount is public calldata.
    /// Charged against the bridge's burning limit.
    function burn(address from, uint256 amount) external onlyAdminOrBridge {
        _useBurningLimit(amount);
        _burn(from, suint256(amount));
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
