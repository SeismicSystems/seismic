// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title BlindBid — sealed tender, award computed inside the TEE
/// @notice The same tender as the portable build, written with Seismic's shielded types.
///         A bid amount is a `suint256`, so the compiler emits `CSTORE`/`CLOAD` for it and
///         the storage slot is flagged private by itself — no Yul library, no `verbatim`.
///         Compiles with the Seismic compiler (`ssolc`, seismic-solidity).
/// @dev The award compares the confidential amounts inside the enclave, so neither the
///      amounts nor their ordering ever exist in public state. The confidential aggregate
///      lets the buyer prove the total committed without revealing any single bid.
contract BlindBid {
    enum Status {
        Open,
        Awarded,
        Cancelled
    }

    error NotBuyer();
    error NotOwner();
    error TenderClosed();
    error TooEarly();
    error TooLate();
    error NoBids();
    error ZeroAmount();

    struct Tender {
        address buyer;
        string title;
        string memo;
        uint256 budget;
        uint64 deadline;
        uint32 bidCount;
        address winner;
        uint256 winningAmount;
        bool priceRevealed;
        bool sealedPrice;
        Status status;
    }

    Tender[] public tenders;

    /// @dev Confidential bid amounts: one flagged storage slot per bid.
    mapping(uint256 => mapping(uint32 => suint256)) private _bidAmount;
    /// @dev Confidential aggregate of every bid filed on a tender.
    mapping(uint256 => suint256) private _total;
    /// @dev The supplier behind each envelope. Public identity, confidential price.
    mapping(uint256 => mapping(uint32 => address)) public bidOwner;

    event TenderCreated(uint256 indexed id, address indexed buyer, uint256 budget, uint64 deadline, bool sealedPrice);
    event BidSubmitted(uint256 indexed tenderId, uint32 indexed bidIndex, address indexed bidder);
    event Awarded(uint256 indexed tenderId, address indexed winner, uint256 revealedAmount);
    event TenderCancelled(uint256 indexed tenderId);
    event TenderRenamed(uint256 indexed tenderId);

    // --------------------------------------------------------------- заказчик

    function createTender(
        string calldata title,
        string calldata memo,
        uint256 budget,
        uint64 deadline,
        bool sealedPrice
    ) external returns (uint256 id) {
        require(deadline > block.timestamp, "deadline in the past");
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

    function cancelTender(uint256 tenderId) external {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        if (t.status != Status.Open) revert TenderClosed();
        t.status = Status.Cancelled;
        emit TenderCancelled(tenderId);
    }

    /// @notice Правка заголовка/описания — пока закупка открыта.
    function renameTender(uint256 tenderId, string calldata newTitle, string calldata newMemo) external {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        if (t.status != Status.Open) revert TenderClosed();
        t.title = newTitle;
        t.memo = newMemo;
        emit TenderRenamed(tenderId);
    }

    // --------------------------------------------------------------- поставщик

    /// @notice Подать запечатанную заявку. Сумма попадает в приватный слот.
    function submitBid(uint256 tenderId, uint256 amount) external returns (uint32 idx) {
        Tender storage t = tenders[tenderId];
        if (t.status != Status.Open) revert TenderClosed();
        if (block.timestamp >= t.deadline) revert TooLate();
        if (amount == 0) revert ZeroAmount();

        idx = t.bidCount;
        _bidAmount[tenderId][idx] = suint256(amount);
        bidOwner[tenderId][idx] = msg.sender;
        t.bidCount = idx + 1;
        _total[tenderId] = suint256(uint256(_total[tenderId]) + amount);

        emit BidSubmitted(tenderId, idx, msg.sender);
    }

    // --------------------------------------------------------------- award

    /// @notice Открыть конверты и выбрать победителя. Сравнение — внутри TEE.
    function closeAndAward(uint256 tenderId) external {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        if (t.status != Status.Open) revert TenderClosed();
        if (block.timestamp < t.deadline) revert TooEarly();

        uint32 n = t.bidCount;
        if (n == 0) revert NoBids();

        uint32 best = 0;
        for (uint32 i = 1; i < n; i++) {
            if (uint256(_bidAmount[tenderId][i]) < uint256(_bidAmount[tenderId][best])) best = i;
        }

        t.winner = bidOwner[tenderId][best];
        if (!t.sealedPrice) {
            t.winningAmount = uint256(_bidAmount[tenderId][best]);
            t.priceRevealed = true;
        }
        t.status = Status.Awarded;

        emit Awarded(tenderId, t.winner, t.winningAmount);
    }

    // --------------------------------------------------------------- чтения

    function tenderCount() external view returns (uint256) {
        return tenders.length;
    }

    function tender(uint256 tenderId) external view returns (Tender memory) {
        return tenders[tenderId];
    }

    /// @notice Своя заявка — видит только её автор.
    function myBid(uint256 tenderId, uint32 idx) external view returns (uint256) {
        if (bidOwner[tenderId][idx] != msg.sender) revert NotOwner();
        return uint256(_bidAmount[tenderId][idx]);
    }

    /// @notice Сумма конкретной заявки: автору или покупателю (после закрытия окна).
    function bidAmount(uint256 tenderId, uint32 idx) external view returns (uint256) {
        Tender storage t = tenders[tenderId];
        if (msg.sender != bidOwner[tenderId][idx] && msg.sender != t.buyer) revert NotOwner();
        if (msg.sender == t.buyer && block.timestamp < t.deadline) revert TooEarly();
        return uint256(_bidAmount[tenderId][idx]);
    }

    /// @notice Все заявки лота — только покупателю и только после закрытия окна.
    /// @dev Требует подписанного чтения: на Seismic `eth_call` не применяет `from`.
    function tenderBids(uint256 tenderId) external view returns (address[] memory bidders, uint256[] memory amounts) {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        if (block.timestamp < t.deadline) revert TooEarly();

        uint32 n = t.bidCount;
        bidders = new address[](n);
        amounts = new uint256[](n);
        for (uint32 i = 0; i < n; i++) {
            bidders[i] = bidOwner[tenderId][i];
            amounts[i] = uint256(_bidAmount[tenderId][i]);
        }
    }

    /// @notice Агрегат заявок для регулятора: сумма без раскрытия отдельных ставок.
    function totalCommitted(uint256 tenderId) external view returns (uint256 total) {
        Tender storage t = tenders[tenderId];
        if (msg.sender != t.buyer) revert NotBuyer();
        total = uint256(_total[tenderId]);
    }
}
