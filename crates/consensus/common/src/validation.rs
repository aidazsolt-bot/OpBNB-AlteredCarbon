//! Collection of methods for block validation.

use alloy_consensus::{BlockHeader as _, EMPTY_OMMER_ROOT_HASH};
use alloy_eips::{eip4844::DATA_GAS_PER_BLOB, eip7840::BlobParams};
use alloy_primitives::B256;
use reth_chainspec::{EthChainSpec, EthereumHardfork, EthereumHardforks};
use reth_consensus::ConsensusError;
use reth_primitives_traits::{
    constants::{GAS_LIMIT_BOUND_DIVISOR, MAXIMUM_GAS_LIMIT_BLOCK, MINIMUM_GAS_LIMIT},
    Block, BlockBody, BlockHeader, GotExpected, SealedBlock, SealedHeader,
};

/// The maximum RLP length of a block, defined in [EIP-7934](https://eips.ethereum.org/EIPS/eip-7934).
///
/// Calculated as `MAX_BLOCK_SIZE` - `SAFETY_MARGIN` where
/// `MAX_BLOCK_SIZE` = `10_485_760`
/// `SAFETY_MARGIN` = `2_097_152`
pub const MAX_RLP_BLOCK_SIZE: usize = 8_388_608;

/// Gas used needs to be less than gas limit. Gas used is going to be checked after execution.
#[inline]
pub fn validate_header_gas<H: BlockHeader>(header: &H) -> Result<(), ConsensusError> {
    if header.gas_used() > header.gas_limit() {
        return Err(ConsensusError::HeaderGasUsedExceedsGasLimit {
            gas_used: header.gas_used(),
            gas_limit: header.gas_limit(),
        })
    }
    // Check that the gas limit is below the maximum allowed gas limit
    if header.gas_limit() > MAXIMUM_GAS_LIMIT_BLOCK {
        return Err(ConsensusError::HeaderGasLimitExceedsMax { gas_limit: header.gas_limit() })
    }
    Ok(())
}

/// Ensure the EIP-1559 base fee is set if the London hardfork is active.
#[inline]
pub fn validate_header_base_fee<H: BlockHeader, ChainSpec: EthereumHardforks>(
    header: &H,
    chain_spec: &ChainSpec,
) -> Result<(), ConsensusError> {
    if chain_spec.is_london_active_at_block(header.number()) && header.base_fee_per_gas().is_none()
    {
        return Err(ConsensusError::BaseFeeMissing)
    }
    Ok(())
}

/// Validate that withdrawals are present in Shanghai
///
/// See [EIP-4895]: Beacon chain push withdrawals as operations
///
/// [EIP-4895]: https://eips.ethereum.org/EIPS/eip-4895
#[inline]
pub fn validate_shanghai_withdrawals<B: Block>(
    block: &SealedBlock<B>,
) -> Result<(), ConsensusError> {
    let withdrawals = block.body().withdrawals().ok_or(ConsensusError::BodyWithdrawalsMissing)?;
    let withdrawals_root = alloy_consensus::proofs::calculate_withdrawals_root(withdrawals);
    let header_withdrawals_root =
        block.withdrawals_root().ok_or(ConsensusError::WithdrawalsRootMissing)?;
    if withdrawals_root != *header_withdrawals_root {
        return Err(ConsensusError::BodyWithdrawalsRootDiff(
            GotExpected { got: withdrawals_root, expected: header_withdrawals_root }.into(),
        ));
    }
    Ok(())
}

/// Validate that blob gas is present in the block if Cancun is active.
///
/// See [EIP-4844]: Shard Blob Transactions
///
/// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
#[inline]
pub fn validate_cancun_gas<B: Block>(block: &SealedBlock<B>) -> Result<(), ConsensusError> {
    // Check that the blob gas used in the header matches the sum of the blob gas used by each
    // blob tx
    let header_blob_gas_used = block.blob_gas_used().ok_or(ConsensusError::BlobGasUsedMissing)?;
    let total_blob_gas = block.body().blob_gas_used();
    if total_blob_gas != header_blob_gas_used {
        return Err(ConsensusError::BlobGasUsedDiff(GotExpected {
            got: header_blob_gas_used,
            expected: total_blob_gas,
        }));
    }
    Ok(())
}

/// Ensures the block response data matches the header.
///
/// This ensures the body response items match the header's hashes:
///   - ommer hash
///   - transaction root
///   - withdrawals root
pub fn validate_body_against_header<B, H>(body: &B, header: &H) -> Result<(), ConsensusError>
where
    B: BlockBody,
    H: BlockHeader,
{
    let ommers_hash = body.calculate_ommers_root();
    if Some(header.ommers_hash()) != ommers_hash {
        return Err(ConsensusError::BodyOmmersHashDiff(
            GotExpected {
                got: ommers_hash.unwrap_or(EMPTY_OMMER_ROOT_HASH),
                expected: header.ommers_hash(),
            }
            .into(),
        ))
    }

    let tx_root = body.calculate_tx_root();
    if header.transactions_root() != tx_root {
        return Err(ConsensusError::BodyTransactionRootDiff(
            GotExpected { got: tx_root, expected: header.transactions_root() }.into(),
        ))
    }

    match (header.withdrawals_root(), body.calculate_withdrawals_root()) {
        (Some(header_withdrawals_root), Some(withdrawals_root)) => {
            if withdrawals_root != header_withdrawals_root {
                return Err(ConsensusError::BodyWithdrawalsRootDiff(
                    GotExpected { got: withdrawals_root, expected: header_withdrawals_root }.into(),
                ))
            }
        }
        (None, None) => {
            // this is ok because we assume the fork is not active in this case
        }
        _ => return Err(ConsensusError::WithdrawalsRootUnexpected),
    }

    Ok(())
}

/// Validate a block without regard for state:
///
/// - Compares the ommer hash in the block header to the block body
/// - Compares the transactions root in the block header to the block body
/// - Pre-execution transaction validation
pub fn validate_block_pre_execution<B, ChainSpec>(
    block: &SealedBlock<B>,
    chain_spec: &ChainSpec,
) -> Result<(), ConsensusError>
where
    B: Block,
    ChainSpec: EthChainSpec + EthereumHardforks,
{
    validate_block_pre_execution_with_tx_root(block, chain_spec, None)
}

/// Validate a block without regard for state using an optional pre-computed transaction root.
///
/// - Compares the ommer hash in the block header to the block body
/// - Compares the transactions root in the block header to the block body
/// - Pre-execution transaction validation
///
/// If `transaction_root` is provided, it is used instead of recomputing the transaction trie
/// root from the block body. The caller must ensure this value was derived from
/// `block.body().calculate_tx_root()`.
pub fn validate_block_pre_execution_with_tx_root<B, ChainSpec>(
    block: &SealedBlock<B>,
    chain_spec: &ChainSpec,
    transaction_root: Option<B256>,
) -> Result<(), ConsensusError>
where
    B: Block,
    ChainSpec: EthChainSpec + EthereumHardforks,
{
    post_merge_hardfork_fields(block, chain_spec)?;

    // Check transaction root
    let expected_transaction_root = block.header().transactions_root();
    let calculated_transaction_root =
        transaction_root.unwrap_or_else(|| block.body().calculate_tx_root());
    if calculated_transaction_root != expected_transaction_root {
        return Err(ConsensusError::BodyTransactionRootDiff(
            GotExpected { got: calculated_transaction_root, expected: expected_transaction_root }
                .into(),
        ))
    }

    Ok(())
}

/// Validates the ommers hash and other fork-specific fields.
///
/// These fork-specific validations are:
/// * EIP-4895 withdrawals validation, if shanghai is active based on the given chainspec. See more
///   information about the specific checks in [`validate_shanghai_withdrawals`].
/// * EIP-4844 blob gas validation, if cancun is active based on the given chainspec. See more
///   information about the specific checks in [`validate_cancun_gas`].
/// * EIP-7934 block size limit validation, if osaka is active based on the given chainspec.
pub fn post_merge_hardfork_fields<B, ChainSpec>(
    block: &SealedBlock<B>,
    chain_spec: &ChainSpec,
) -> Result<(), ConsensusError>
where
    B: Block,
    ChainSpec: EthereumHardforks,
{
    // Check ommers hash
    let ommers_hash = block.body().calculate_ommers_root();
    if Some(block.ommers_hash()) != ommers_hash {
        return Err(ConsensusError::BodyOmmersHashDiff(
            GotExpected {
                got: ommers_hash.unwrap_or(EMPTY_OMMER_ROOT_HASH),
                expected: block.ommers_hash(),
            }
            .into(),
        ))
    }

    // EIP-4895: Beacon chain push withdrawals as operations
    if chain_spec.is_shanghai_active_at_timestamp(block.timestamp()) {
        validate_shanghai_withdrawals(block)?;
    }

    if chain_spec.is_cancun_active_at_timestamp(block.timestamp()) {
        validate_cancun_gas(block)?;
    }

    if chain_spec.is_osaka_active_at_timestamp(block.timestamp()) &&
        block.rlp_length() > MAX_RLP_BLOCK_SIZE
    {
        return Err(ConsensusError::BlockTooLarge {
            rlp_length: block.rlp_length(),
            max_rlp_length: MAX_RLP_BLOCK_SIZE,
        })
    }

    Ok(())
}

/// Validates that the EIP-4844 header fields exist and conform to the spec. This ensures that:
///
///  * `blob_gas_used` exists as a header field
///  * `parent_beacon_block_root` exists as a header field
///  * `blob_gas_used` is a multiple of `DATA_GAS_PER_BLOB`
///  * `blob_gas_used` doesn't exceed the max allowed blob gas based on the given params
pub fn validate_4844_header_standalone<H: BlockHeader>(
    header: &H,
    blob_params: BlobParams,
) -> Result<(), ConsensusError> {
    let blob_gas_used = header.blob_gas_used().ok_or(ConsensusError::BlobGasUsedMissing)?;

    if header.parent_beacon_block_root().is_none() {
        return Err(ConsensusError::ParentBeaconBlockRootMissing)
    }

    if !blob_gas_used.is_multiple_of(DATA_GAS_PER_BLOB) {
        return Err(ConsensusError::BlobGasUsedNotMultipleOfBlobGasPerBlob {
            blob_gas_used,
            blob_gas_per_blob: DATA_GAS_PER_BLOB,
        })
    }

    if blob_gas_used > blob_params.max_blob_gas_per_block() {
        return Err(ConsensusError::BlobGasUsedExceedsMaxBlobGasPerBlock {
            blob_gas_used,
            max_blob_gas_per_block: blob_params.max_blob_gas_per_block(),
        })
    }

    Ok(())
}

/// Validates the header's extra data according to the beacon consensus rules.
///
/// From yellow paper: extraData: An arbitrary byte array containing data relevant to this block.
/// This must be 32 bytes or fewer; formally Hx.
#[inline]
pub fn validate_header_extra_data<H: BlockHeader>(
    header: &H,
    max_size: usize,
) -> Result<(), ConsensusError> {
    let extra_data_len = header.extra_data().len();
    if extra_data_len > max_size {
        Err(ConsensusError::ExtraDataExceedsMax { len: extra_data_len })
    } else {
        Ok(())
    }
}

/// Validates against the parent hash and number.
///
/// This function ensures that the header block number is sequential and that the hash of the parent
/// header matches the parent hash in the header.
#[inline]
pub fn validate_against_parent_hash_number<H: BlockHeader>(
    header: &H,
    parent: &SealedHeader<H>,
) -> Result<(), ConsensusError> {
    if parent.hash() != header.parent_hash() {
        return Err(ConsensusError::ParentHashMismatch(
            GotExpected { got: header.parent_hash(), expected: parent.hash() }.into(),
        ))
    }

    let Some(parent_number) = parent.number().checked_add(1) else {
        // parent block already reached the maximum
        return Err(ConsensusError::ParentBlockNumberMismatch {
            parent_block_number: parent.number(),
            block_number: u64::MAX,
        })
    };

    // Parent number is consistent.
    if parent_number != header.number() {
        return Err(ConsensusError::ParentBlockNumberMismatch {
            parent_block_number: parent.number(),
            block_number: header.number(),
        })
    }

    Ok(())
}

/// Validates the base fee against the parent and EIP-1559 rules.
#[inline]
pub fn validate_against_parent_eip1559_base_fee<ChainSpec: EthChainSpec + EthereumHardforks>(
    header: &ChainSpec::Header,
    parent: &ChainSpec::Header,
    chain_spec: &ChainSpec,
) -> Result<(), ConsensusError> {
    if chain_spec.is_london_active_at_block(header.number()) {
        let base_fee = header.base_fee_per_gas().ok_or(ConsensusError::BaseFeeMissing)?;

        let expected_base_fee = if chain_spec
            .ethereum_fork_activation(EthereumHardfork::London)
            .transitions_at_block(header.number())
        {
            alloy_eips::eip1559::INITIAL_BASE_FEE
        } else {
            chain_spec
                .next_block_base_fee(parent, header.timestamp())
                .ok_or(ConsensusError::BaseFeeMissing)?
        };
        if expected_base_fee != base_fee {
            return Err(ConsensusError::BaseFeeDiff(GotExpected {
                expected: expected_base_fee,
                got: base_fee,
            }))
        }
    }

    Ok(())
}

/// Validates that the block timestamp is greater than the parent block timestamp.
#[inline]
pub fn validate_against_parent_timestamp<H: BlockHeader>(
    header: &H,
    parent: &H,
) -> Result<(), ConsensusError> {
    if header.timestamp() <= parent.timestamp() {
        return Err(ConsensusError::TimestampIsInPast {
            parent_timestamp: parent.timestamp(),
            timestamp: header.timestamp(),
        })
    }
    Ok(())
}

/// Validates gas limit against parent gas limit.
///
/// The maximum allowable difference between self and parent gas limits is determined by the
/// parent's gas limit divided by the [`GAS_LIMIT_BOUND_DIVISOR`].
#[inline]
pub fn validate_against_parent_gas_limit<
    H: BlockHeader,
    ChainSpec: EthChainSpec + EthereumHardforks,
>(
    header: &SealedHeader<H>,
    parent: &SealedHeader<H>,
    chain_spec: &ChainSpec,
) -> Result<(), ConsensusError> {
    // Determine the parent gas limit, considering elasticity multiplier on the London fork.
    let parent_gas_limit = if !chain_spec.is_london_active_at_block(parent.number()) &&
        chain_spec.is_london_active_at_block(header.number())
    {
        parent.gas_limit() *
            chain_spec.base_fee_params_at_timestamp(header.timestamp()).elasticity_multiplier
                as u64
    } else {
        parent.gas_limit()
    };

    // Check for an increase in gas limit beyond the allowed threshold.
    if header.gas_limit() > parent_gas_limit {
        if header.gas_limit() - parent_gas_limit >= parent_gas_limit / GAS_LIMIT_BOUND_DIVISOR {
            return Err(ConsensusError::GasLimitInvalidIncrease {
                parent_gas_limit,
                child_gas_limit: header.gas_limit(),
            })
        }
    }
    // Check for a decrease in gas limit beyond the allowed threshold.
    else if parent_gas_limit - header.gas_limit() >= parent_gas_limit / GAS_LIMIT_BOUND_DIVISOR {
        return Err(ConsensusError::GasLimitInvalidDecrease {
            parent_gas_limit,
            child_gas_limit: header.gas_limit(),
        })
    }
    // Check if the self gas limit is below the minimum required limit.
    if header.gas_limit() < MINIMUM_GAS_LIMIT {
        return Err(ConsensusError::GasLimitInvalidMinimum { child_gas_limit: header.gas_limit() })
    }

    Ok(())
}

/// Validates that the EIP-4844 header fields are correct with respect to the parent block. This
/// ensures that the `blob_gas_used` and `excess_blob_gas` fields exist in the child header, and
/// that the `excess_blob_gas` field matches the expected `excess_blob_gas` calculated from the
/// parent header fields.
pub fn validate_against_parent_4844<H: BlockHeader>(
    header: &H,
    parent: &H,
    blob_params: BlobParams,
) -> Result<(), ConsensusError> {
    // From [EIP-4844](https://eips.ethereum.org/EIPS/eip-4844#header-extension):
    //
    // > For the first post-fork block, both parent.blob_gas_used and parent.excess_blob_gas
    // > are evaluated as 0.
    //
    // This means in the first post-fork block, calc_excess_blob_gas will return 0.
    let parent_blob_gas_used = parent.blob_gas_used().unwrap_or(0);
    let parent_excess_blob_gas = parent.excess_blob_gas().unwrap_or(0);

    if header.blob_gas_used().is_none() {
        return Err(ConsensusError::BlobGasUsedMissing)
    }
    let excess_blob_gas = header.excess_blob_gas().ok_or(ConsensusError::ExcessBlobGasMissing)?;

    let parent_base_fee_per_gas = parent.base_fee_per_gas().unwrap_or(0);
    let expected_excess_blob_gas = blob_params.next_block_excess_blob_gas_osaka(
        parent_excess_blob_gas,
        parent_blob_gas_used,
        parent_base_fee_per_gas,
    );
    if expected_excess_blob_gas != excess_blob_gas {
        return Err(ConsensusError::ExcessBlobGasDiff {
            diff: GotExpected { got: excess_blob_gas, expected: expected_excess_blob_gas },
            parent_excess_blob_gas,
            parent_blob_gas_used,
        })
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::{BlockBody, Header, TxEip4844};
    use alloy_eips::{eip4844::DATA_GAS_PER_BLOB, eip4895::Withdrawals};
    use alloy_primitives::{Address, Bytes, Signature, U256};
    use rand::Rng;
    use reth_chainspec::ChainSpecBuilder;
    use reth_ethereum_primitives::{Transaction, TransactionSigned};
    use reth_primitives_traits::proofs;

    fn mock_blob_tx(nonce: u64, num_blobs: usize) -> TransactionSigned {
        let mut rng = rand::rng();
        let request = Transaction::Eip4844(TxEip4844 {
            chain_id: 1u64,
            nonce,
            max_fee_per_gas: 0x28f000fff,
            max_priority_fee_per_gas: 0x28f000fff,
            max_fee_per_blob_gas: 0x7,
            gas_limit: 10,
            to: Address::default(),
            value: U256::from(3_u64),
            input: Bytes::from(vec![1, 2]),
            access_list: Default::default(),
            blob_versioned_hashes: std::iter::repeat_with(|| rng.random())
                .take(num_blobs)
                .collect(),
        });

        let signature = Signature::new(U256::default(), U256::default(), true);

        TransactionSigned::new_unhashed(request, signature)
    }

    /// got test block
    fn mock_block() -> (SealedBlock, Header) {
        // https://etherscan.io/block/15867168 where transaction root and receipts root are cleared
        // empty merkle tree: 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421

        let header = Header {
            parent_hash: hex!("859fad46e75d9be177c2584843501f2270c7e5231711e90848290d12d7c6dcdd").into(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: hex!("4675c7e5baafbffbca748158becba61ef3b0a263").into(),
            state_root: hex!("8337403406e368b3e40411138f4868f79f6d835825d55fd0c2f6e17b1a3948e9").into(),
            transactions_root: EMPTY_ROOT_HASH,
            receipts_root: EMPTY_ROOT_HASH,
            logs_bloom: hex!("002400000000004000220000800002000000000000000000000000000000100000000000000000100000000000000021020000000800000006000000002100040000000c0004000000000008000008200000000000000000000000008000000001040000020000020000002000000800000002000020000000022010000000000000010002001000000000020200000000000001000200880000004000000900020000000000020000000040000000000000000000000000000080000000000001000002000000000000012000200020000000000000001000000000000020000010321400000000100000000000000000000000000000400000000000000000").into(),
            difficulty: U256::ZERO, // total difficulty: 0xc70d815d562d3cfa955).into(),
            number: 0xf21d20,
            gas_limit: 0x1c9c380,
            gas_used: 0x6e813,
            timestamp: 0x635f9657,
            extra_data: hex!("")[..].into(),
            mix_hash: hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
            nonce: 0x0000000000000000u64.into(),
            base_fee_per_gas: 0x28f0001df.into(),
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
            requests_hash: None
        };
        // size: 0x9b5

        let mut parent = header.clone();
        parent.gas_used = 17763076;
        parent.gas_limit = 30000000;
        parent.base_fee_per_gas = Some(0x28041f7f5);
        parent.number -= 1;
        parent.timestamp -= 1;

        let ommers = Vec::new();
        let transactions = Vec::new();

        let sealed = header.seal_slow();
        let (header, seal) = sealed.into_parts();

        (
            SealedBlock {
                header: SealedHeader::new(header, seal),
                body: BlockBody { transactions, ommers, withdrawals: None, sidecars: None },
            },
            parent,
        )
    }

    #[test]
    fn valid_withdrawal_index() {
        let chain_spec = ChainSpecBuilder::mainnet().shanghai_activated().build();

        let create_block_with_withdrawals = |indexes: &[u64]| {
            let withdrawals = Withdrawals::new(
                indexes
                    .iter()
                    .map(|idx| Withdrawal { index: *idx, ..Default::default() })
                    .collect(),
            );

            let sealed = Header {
                withdrawals_root: Some(proofs::calculate_withdrawals_root(&withdrawals)),
                ..Default::default()
            }
            .seal_slow();
            let (header, seal) = sealed.into_parts();

            SealedBlock {
                header: SealedHeader::new(header, seal),
                body: BlockBody { withdrawals: Some(withdrawals), ..Default::default() },
            }
        };

        // Single withdrawal
        let block = create_block_with_withdrawals(&[1]);
        assert_eq!(validate_block_pre_execution(&block, &chain_spec), Ok(()));

        // Multiple increasing withdrawals
        let block = create_block_with_withdrawals(&[1, 2, 3]);
        assert_eq!(validate_block_pre_execution(&block, &chain_spec), Ok(()));
        let block = create_block_with_withdrawals(&[5, 6, 7, 8, 9]);
        assert_eq!(validate_block_pre_execution(&block, &chain_spec), Ok(()));
        let (_, parent) = mock_block();

        // Withdrawal index should be the last withdrawal index + 1
        let mut provider = Provider::new(Some(parent));
        provider
            .withdrawals_provider
            .expect_latest_withdrawal()
            .return_const(Ok(Some(Withdrawal { index: 2, ..Default::default() })));
    }

    #[test]
    fn cancun_block_incorrect_blob_gas_used() {
        let chain_spec = ChainSpecBuilder::mainnet().cancun_activated().build();

        // create a tx with 10 blobs
        let transaction = mock_blob_tx(1, 10);

        let header = Header {
            base_fee_per_gas: Some(1337),
            withdrawals_root: Some(proofs::calculate_withdrawals_root(&[])),
            blob_gas_used: Some(1),
            transactions_root: proofs::calculate_transaction_root(std::slice::from_ref(
                &transaction,
            )),
            ..Default::default()
        };
        let body = BlockBody {
            transactions: vec![transaction],
            ommers: vec![],
            withdrawals: Some(Withdrawals::default()),
            sidecars: None,
        };

        let block = SealedBlock::seal_slow(alloy_consensus::Block { header, body });

        // 10 blobs times the blob gas per blob.
        let expected_blob_gas_used = 10 * DATA_GAS_PER_BLOB;

        // validate blob, it should fail blob gas used validation
        assert!(matches!(
            validate_block_pre_execution(&block, &chain_spec).unwrap_err(),
            ConsensusError::BlobGasUsedDiff(diff)
                if diff.got == 1 && diff.expected == expected_blob_gas_used
        ));
    }

    #[test]
    fn validate_header_extra_data_with_custom_limit() {
        // Test with default 32 bytes - should pass
        let header_32 = Header { extra_data: Bytes::from(vec![0; 32]), ..Default::default() };
        assert!(validate_header_extra_data(&header_32, 32).is_ok());

        // Test exceeding default - should fail
        let header_33 = Header { extra_data: Bytes::from(vec![0; 33]), ..Default::default() };
        assert!(matches!(
            validate_header_extra_data(&header_33, 32).unwrap_err(),
            ConsensusError::ExtraDataExceedsMax { len } if len == 33
        ));

        // Test with custom larger limit - should pass
        assert!(validate_header_extra_data(&header_33, 64).is_ok());
    }

    #[test]
    fn precomputed_tx_root_correct_passes() {
        let chain_spec = ChainSpecBuilder::mainnet().cancun_activated().build();

        let transaction = mock_blob_tx(1, 1);
        let tx_root = proofs::calculate_transaction_root(std::slice::from_ref(&transaction));

        let header = Header {
            base_fee_per_gas: Some(1337),
            withdrawals_root: Some(proofs::calculate_withdrawals_root(&[])),
            transactions_root: tx_root,
            blob_gas_used: Some(DATA_GAS_PER_BLOB),
            excess_blob_gas: Some(0),
            ..Default::default()
        };
        let body = BlockBody {
            transactions: vec![transaction],
            ommers: vec![],
            withdrawals: Some(Withdrawals::default()),
        };

        let block = SealedBlock::seal_slow(alloy_consensus::Block { header, body });

        // Some(correct_root) should pass just like None
        assert!(
            validate_block_pre_execution_with_tx_root(&block, &chain_spec, Some(tx_root)).is_ok()
        );
        assert!(validate_block_pre_execution_with_tx_root(&block, &chain_spec, None).is_ok());
    }

    #[test]
    fn precomputed_tx_root_wrong_fails() {
        let chain_spec = ChainSpecBuilder::mainnet().cancun_activated().build();

        let transaction = mock_blob_tx(1, 1);
        let tx_root = proofs::calculate_transaction_root(std::slice::from_ref(&transaction));

        let header = Header {
            base_fee_per_gas: Some(1337),
            withdrawals_root: Some(proofs::calculate_withdrawals_root(&[])),
            transactions_root: tx_root,
            blob_gas_used: Some(DATA_GAS_PER_BLOB),
            excess_blob_gas: Some(0),
            ..Default::default()
        };
        let body = BlockBody {
            transactions: vec![transaction],
            ommers: vec![],
            withdrawals: Some(Withdrawals::default()),
        };

        let block = SealedBlock::seal_slow(alloy_consensus::Block { header, body });

        let wrong_root = B256::repeat_byte(0xff);
        assert!(matches!(
            validate_block_pre_execution_with_tx_root(&block, &chain_spec, Some(wrong_root))
                .unwrap_err(),
            ConsensusError::BodyTransactionRootDiff(diff)
                if diff.0.got == wrong_root && diff.0.expected == tx_root
        ));
    }
}
