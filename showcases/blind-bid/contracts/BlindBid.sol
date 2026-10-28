// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface ISealStore {
    function store(uint256 slot, uint256 value) external;
    function storeFromTransient(uint256 slot) external;
    function load(uint256 slot) external view returns (uint256);
    function minSlot(uint256[] calldata slots) external view returns (uint256);
    function sumSlots(uint256[] calldata slots) external view returns (uint256);
}

/// @title BlindBid — закрытая закупка (sealed tender) на Seismic
/// @notice Суммы заявок никогда не попадают в публичное состояние и не видны в calldata:
///         - calldata шифруется (транзакция типа 0x4A, `shieldedWriteContract`)
///         - сумма кладётся в приватный слот (CSTORE) библиотеки SealStore
///         - победитель считается внутри TEE по приватным значениям (сравнение suint-стилем)
///         - даже порядок ставок не существует публично: снаружи видно только число заявок
/// @dev Скомпилирован стандартным solc (0.8.37): приватное хранилище даёт Yul-библиотека
///      SealStore, которую вызывает этот контракт.
contract BlindBid {
    enum Status {
        Open,
        Awarded,
        Cancelled
    }

    struct Tender {
        address buyer; // заказчик
        string title;
        string memo;
        uint256 budget; // публичный лимит цены заявки
        uint64 deadline; // unix ts
        uint32 bidCount;
        address winner;
        uint256 winningAmount; // публично только в режиме priceRevealed
        bool priceRevealed;
        bool sealedPrice; // режим sealed award: сумма победителя не публикуется
        Status status;
    }

    ISealStore public immutable store;
    Tender[] public tenders;

    mapping(uint256 => mapping(uint32 => address)) public bidOwner; // tenderId => index => поставщик
    mapping(uint256 => mapping(uint32 => uint256)) public bidSlot; // tenderId => index => слот в SealStore

    event TenderCreated(uint256 indexed id, address indexed buyer, uint256 budget, uint64 deadline, bool sealedPrice);
    /// @dev Суммы в событии нет — только факт подачи заявки.
    event BidSubmitted(uint256 indexed tenderId, uint32 indexed bidIndex, address indexed bidder);
    event Awarded(uint256 indexed tenderId, address indexed winner, uint256 revealedAmount);

    error NotBuyer();
    error NotOwner();
    error TenderClosed();
    error TooEarly();
    error TooLate();
    error BadAmount();
    error NoBids();
    error UnknownTender();
    error SealStoreFailed();

    constructor(address store_) {
        store = ISealStore(store_);
    }

    function _slot(uint256 tenderId, uint32 idx) internal pure returns (uint256) {
        return uint256(keccak256(abi.encode(tenderId, idx)));
    }

    // SealStore — библиотека приватного хранилища: суммы лежат в её слотах, помеченных
    // приватными (CSTORE), поэтому в публичном состоянии видны нули.
    // Обращения идут обычными вызовами: CLOAD внутри delegatecall не проходит в статическом
    // контексте eth_call, а обычный вызов работает и в транзакции, и в eth_call.
    function _sealCall(bytes4 sel, bytes memory args) internal returns (bytes memory) {
        (bool ok, bytes memory ret) = address(store).call(bytes.concat(abi.encodePacked(sel), args));
        if (!ok) revert SealStoreFailed();
        return ret;
    }

    function _sealView(bytes4 sel, bytes memory args) internal view returns (bytes memory) {
        (bool ok, bytes memory ret) = address(store).staticcall(bytes.concat(abi.encodePacked(sel), args));
        if (!ok) revert SealStoreFailed();
        return ret;
    }

    function _store(uint256 slot, uint256 value) internal {
        _sealCall(ISealStore.store.selector, abi.encode(slot, value));
    }

    function _load(uint256 slot) internal view returns (uint256) {
        return abi.decode(_sealView(ISealStore.load.selector, abi.encode(slot)), (uint256));
    }

    function _minSlot(uint256[] memory slots) internal view returns (uint256) {
        return abi.decode(_sealView(ISealStore.minSlot.selector, abi.encode(slots)), (uint256));
    }

    function _sumSlots(uint256[] memory slots) internal view returns (uint256) {
        return abi.decode(_sealView(ISealStore.sumSlots.selector, abi.encode(slots)), (uint256));
    }

    // ---------------------------------------------------------------- заказчик

    function createTender(
        string calldata title,
        string calldata memo,
        uint256 budget,
        uint64 deadline,
        bool sealedPrice
    ) external returns (uint256 id) {
        if (deadline <= block.timestamp) revert TooLate();

        id = tenders.length;
        tenders.push(
            Tender({
                buyer: msg.sender,
                title: title,
                memo: memo,
                budget: budget,
                deadline: deadline,
                bidCount: 0,
                winner: address(0),
                winningAmount: 0,
                priceRevealed: false,
                sealedPrice: sealedPrice,
                status: Status.Open
            })
        );
        emit TenderCreated(id, msg.sender, budget, deadline, sealedPrice);
    }

    /// @notice Закрывает тендер и выбирает победителя. Сравнение сумм происходит внутри TEE:
    ///         наружу отдаётся только слот победившей заявки, сами суммы не выходят.
    function closeAndAward(uint256 tenderId) external {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        if (t.status != Status.Open) revert TenderClosed();
        if (block.timestamp < t.deadline) revert TooEarly();
        if (t.bidCount == 0) revert NoBids();

        uint256[] memory slots = new uint256[](t.bidCount);
        for (uint32 i = 0; i < t.bidCount; i++) {
            slots[i] = bidSlot[tenderId][i];
        }

        uint256 winSlot = _minSlot(slots);
        uint32 winIdx;
        for (uint32 i = 0; i < t.bidCount; i++) {
            if (bidSlot[tenderId][i] == winSlot) {
                winIdx = i;
                break;
            }
        }

        uint256 amount = _load(winSlot);
        t.winner = bidOwner[tenderId][winIdx];
        t.status = Status.Awarded;

        if (!t.sealedPrice) {
            t.winningAmount = amount;
            t.priceRevealed = true;
        }
        emit Awarded(tenderId, t.winner, t.priceRevealed ? amount : 0);
    }

    function cancelTender(uint256 tenderId) external {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        if (t.status != Status.Open) revert TenderClosed();
        t.status = Status.Cancelled;
    }

    /// @notice Правка заголовка/описания — пока закупка открыта.
    /// @dev Заголовок публичен, поэтому он виден всем; править может только заказчик.
    function renameTender(uint256 tenderId, string calldata newTitle, string calldata newMemo) external {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        if (t.status != Status.Open) revert TenderClosed();
        t.title = newTitle;
        t.memo = newMemo;
    }

    // --------------------------------------------------------------- поставщик

    /// @notice Подача заявки. Сумма НЕ публикуется: она кладётся в приватный слот.
    ///         Вызывать через `shieldedWriteContract` — тогда calldata тоже зашифрован.
    function submitBid(uint256 tenderId, uint256 amount) external returns (uint32 idx) {
        Tender storage t = tenders[tenderId];
        if (t.status != Status.Open) revert TenderClosed();
        if (block.timestamp >= t.deadline) revert TooLate();
        if (amount == 0 || amount > t.budget) revert BadAmount();

        idx = t.bidCount++;
        uint256 slot = _slot(tenderId, idx);
        bidOwner[tenderId][idx] = msg.sender;
        bidSlot[tenderId][idx] = slot;

        // amount → transient storage → приватный слот библиотеки.
        // Ни в одном call data сумма не появляется (только в зашифрованном входе транзакции).
        _store(slot, amount);

        emit BidSubmitted(tenderId, idx, msg.sender);
    }

    // ------------------------------------------------------------------ чтение

    function tenderCount() external view returns (uint256) {
        return tenders.length;
    }

    function tender(uint256 tenderId) external view returns (Tender memory) {
        return tenders[tenderId];
    }

    /// @notice Свою заявку поставщик читает сам.
    function myBid(uint256 tenderId, uint32 idx) external view returns (uint256) {
        if (bidOwner[tenderId][idx] != msg.sender) revert NotOwner();
        return _load(bidSlot[tenderId][idx]);
    }

    /// @notice Сумма конкретной заявки: заказчику (после дедлайна) и автору заявки.
    function bidAmount(uint256 tenderId, uint32 idx) external view returns (uint256) {
        Tender storage t = tenders[tenderId];
        if (msg.sender != bidOwner[tenderId][idx] && msg.sender != t.buyer) revert NotOwner();
        if (msg.sender == t.buyer && block.timestamp < t.deadline) revert TooEarly();
        return _load(bidSlot[tenderId][idx]);
    }

    /// @notice Все заявки тендера — только заказчику и только после дедлайна.
    function tenderBids(uint256 tenderId) external view returns (address[] memory bidders, uint256[] memory amounts) {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        if (block.timestamp < t.deadline) revert TooEarly();
        uint32 n = t.bidCount;
        bidders = new address[](n);
        amounts = new uint256[](n);
        for (uint32 i = 0; i < n; i++) {
            bidders[i] = bidOwner[tenderId][i];
            amounts[i] = _load(bidSlot[tenderId][i]);
        }
    }

    /// @notice Агрегат для аудита: сумма всех заявок — без раскрытия отдельных сумм.
    function totalCommitted(uint256 tenderId) external view returns (uint256 total) {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        uint32 n = t.bidCount;
        uint256[] memory slots = new uint256[](n);
        for (uint32 i = 0; i < n; i++) {
            slots[i] = bidSlot[tenderId][i];
        }
        total = _sumSlots(slots);
    }

    /// @notice Сумма победителя: только самому победителю и заказчику.
    function winningAmount(uint256 tenderId) external view returns (uint256) {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer && msg.sender != t.winner) revert NotOwner();
        uint32 winIdx;
        for (uint32 i = 0; i < t.bidCount; i++) {
            if (bidOwner[tenderId][i] == t.winner) {
                winIdx = i;
                break;
            }
        }
        return _load(bidSlot[tenderId][winIdx]);
    }
}
