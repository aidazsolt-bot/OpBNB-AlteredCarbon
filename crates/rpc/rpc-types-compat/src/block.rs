use alloy_consensus::BlockHeader as _;
use alloy_primitives::{B256, U256};
use alloy_rlp::Encodable;
use alloy_rpc_types::{Block as RpcBlock, BlockTransactions, BlockTransactionsKind};
use alloy_rpc_types_eth::{BlockError, Header, TransactionInfo};
use reth_primitives::{Block, BlockWithSenders, Header as PrimitiveHeader, SealedHeader};
use reth_primitives_traits::BlockBody as _;

use crate::{transaction::from_recovered_with_block_context, TransactionCompat};

pub fn from_block<T: TransactionCompat>(
    block: BlockWithSenders,
    total_difficulty: U256,
    kind: BlockTransactionsKind,
    block_hash: Option<B256>,
    tx_resp_builder: &T,
) -> Result<RpcBlock<T::Transaction>, BlockError> {
    match kind {
        BlockTransactionsKind::Hashes => Ok(from_block_with_tx_hashes::<T::Transaction>(
            block,
            total_difficulty,
            block_hash,
        )),
        BlockTransactionsKind::Full => {
            from_block_full::<T>(block, total_difficulty, block_hash, tx_resp_builder)
        }
    }
}

pub fn from_block_with_tx_hashes<T>(
    block: BlockWithSenders,
    total_difficulty: U256,
    block_hash: Option<B256>,
) -> RpcBlock<T> {
    let block_hash = block_hash.unwrap_or_else(|| block.hash());
    let transactions = block.body().transactions_iter().map(|tx| *tx.tx_hash()).collect();

    from_block_with_transactions(
        block.rlp_length(),
        block_hash,
        block.into_block(),
        total_difficulty,
        BlockTransactions::Hashes(transactions),
    )
}

pub fn from_block_full<T: TransactionCompat>(
    block: BlockWithSenders,
    total_difficulty: U256,
    block_hash: Option<B256>,
    tx_resp_builder: &T,
) -> Result<RpcBlock<T::Transaction>, BlockError> {
    let block_hash = block_hash.unwrap_or_else(|| block.hash());
    let block_number = block.header().number();
    let base_fee_per_gas = block.header().base_fee_per_gas();
    let block_timestamp = block.header().timestamp();
    let block_length = block.rlp_length();

    let transactions = block
        .clone()
        .into_transactions_recovered()
        .enumerate()
        .map(|(idx, tx)| {
            let tx_hash = *tx.tx_hash();
            let tx_info = TransactionInfo {
                hash: Some(tx_hash),
                block_hash: Some(block_hash),
                block_number: Some(block_number),
                base_fee: base_fee_per_gas,
                index: Some(idx as u64),
                block_timestamp: Some(block_timestamp),
            };

            from_recovered_with_block_context::<T>(tx, tx_info, tx_resp_builder)
        })
        .collect::<Vec<_>>();

    Ok(from_block_with_transactions(
        block_length,
        block_hash,
        block.into_block(),
        total_difficulty,
        BlockTransactions::Full(transactions),
    ))
}

pub fn from_primitive_with_hash(primitive_header: SealedHeader<PrimitiveHeader>) -> Header {
    let (header, hash) = primitive_header.split();
    let PrimitiveHeader {
        parent_hash,
        ommers_hash,
        beneficiary,
        state_root,
        transactions_root,
        receipts_root,
        logs_bloom,
        difficulty,
        number,
        gas_limit,
        gas_used,
        timestamp,
        mix_hash,
        nonce,
        base_fee_per_gas,
        extra_data,
        withdrawals_root,
        blob_gas_used,
        excess_blob_gas,
        parent_beacon_block_root,
        requests_hash,
        block_access_list_hash,
        slot_number,
    } = header;

    Header {
        inner: alloy_consensus::Header {
            parent_hash,
            ommers_hash,
            beneficiary,
            state_root,
            transactions_root,
            receipts_root,
            logs_bloom,
            difficulty,
            number,
            gas_limit,
            gas_used,
            timestamp,
            mix_hash,
            nonce,
            base_fee_per_gas,
            extra_data,
            withdrawals_root,
            blob_gas_used,
            excess_blob_gas,
            parent_beacon_block_root,
            requests_hash,
            block_access_list_hash,
            slot_number,
        },
        hash,
        total_difficulty: None,
        size: None,
    }
}

fn from_block_with_transactions<T>(
    block_length: usize,
    block_hash: B256,
    block: Block,
    total_difficulty: U256,
    transactions: BlockTransactions<T>,
) -> RpcBlock<T> {
    let uncles = block.body.ommers.iter().map(|h| h.hash_slow()).collect();
    let mut header = from_primitive_with_hash(SealedHeader::new(block.header, block_hash));
    header.total_difficulty = Some(total_difficulty);
    header.size = Some(U256::from(block_length));

    RpcBlock::new(header, transactions)
        .with_uncles(uncles)
        .with_withdrawals(block.body.withdrawals)
}

pub fn uncle_block_from_header<T>(header: PrimitiveHeader) -> RpcBlock<T> {
    let hash = header.hash_slow();
    let uncle_block: Block = Block { header, ..Default::default() };
    let size = U256::from(uncle_block.length());
    let mut rpc_header = from_primitive_with_hash(SealedHeader::new(uncle_block.header, hash));
    rpc_header.size = Some(size);
    RpcBlock::new(rpc_header, BlockTransactions::Uncle)
}
