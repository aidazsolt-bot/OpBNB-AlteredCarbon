use crate::{
    add_static_file_mask,
    static_file::mask::{ColumnSelectorOne, ColumnSelectorTwo},
    HeaderTerminalDifficulties, RawValue, Transactions,
};
use alloy_primitives::{Address, BlockHash};
use reth_db_api::AccountChangeSets;
use reth_db_api::table::Table;

// HEADER MASKS
add_static_file_mask! {
    #[doc = "Mask for selecting a single header from Headers static file segment"]
    HeaderMask<H>, H, 0b001
}
add_static_file_mask! {
    #[doc = "Mask for selecting a total difficulty value from Headers static file segment"]
    TotalDifficultyMask, <HeaderTerminalDifficulties as Table>::Value, 0b010
}
add_static_file_mask! {
    #[doc = "Mask for selecting a block hash value from Headers static file segment"]
    BlockHashMask, BlockHash, 0b100
}
add_static_file_mask! {
    #[doc = "Mask for selecting a header along with block hash from Headers static file segment"]
    HeaderWithHashMask<H>, H, BlockHash, 0b101
}
add_static_file_mask! {
    #[doc = "Mask for selecting a total difficulty along with block hash from Headers static file segment"]
    TDWithHashMask,
    <HeaderTerminalDifficulties as Table>::Value,
    BlockHash,
    0b110
}

// RECEIPT MASKS
add_static_file_mask! {
    #[doc = "Mask for selecting a single receipt from `Receipts` static file segment"]
    ReceiptMask<R>, R, 0b1
}

// TRANSACTION MASKS
add_static_file_mask! {
    #[doc = "Mask for selecting a single transaction from `Transactions` static file segment"]
    TransactionMask<T>, T, 0b1
}
add_static_file_mask! {
    #[doc = "Mask for selecting a single raw transaction from `Transactions` static file segment"]
    RawTransactionMask<T>, RawValue<<Transactions as Table>::Value>, 0b1
}

// TRANSACTION SENDER MASKS
add_static_file_mask! {
    #[doc = "Mask for selecting a single transaction sender from `TransactionSenders` static file segment"]
    TransactionSenderMask, Address, 0b1
}

// SIDECAR MASKS
add_static_file_mask! {
    #[doc = "Mask for selecting a single blob sidecar from `Sidecars` static file segment"]
    SidecarMask<S>, S, 0b01
}
add_static_file_mask! {
    #[doc = "Mask for selecting a block hash from `Sidecars` static file segment"]
    SidecarBlockHashMask, BlockHash, 0b10
}
add_static_file_mask! {
    #[doc = "Mask for selecting a blob sidecar together with block hash from `Sidecars` static file segment"]
    SidecarWithHashMask<S>, S, BlockHash, 0b11
}

// ACCOUNT CHANGESET MASKS
add_static_file_mask! {
    #[doc = "Mask for selecting a single changeset from `AccountChangesets` static file segment"]
    AccountChangesetMask, <AccountChangeSets as reth_db_api::table::Table>::Value, 0b1
}
