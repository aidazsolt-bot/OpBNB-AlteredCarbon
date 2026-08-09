//! Optimism Consensus implementation.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::sync::Arc;
use alloy_consensus::{
    constants::MAXIMUM_EXTRA_DATA_SIZE, BlockHeader as _, EMPTY_OMMER_ROOT_HASH,
};
use alloy_primitives::B64;
use core::fmt::Debug;
use reth_chainspec::EthChainSpec;
use reth_consensus::{Consensus, ConsensusError, FullConsensus, HeaderValidator, ReceiptRootBloom};
use reth_consensus_common::validation::{
    validate_against_parent_eip1559_base_fee, validate_against_parent_hash_number,
    validate_against_parent_timestamp, validate_cancun_gas, validate_header_base_fee,
    validate_header_extra_data, validate_header_gas, validate_shanghai_withdrawals,
};
use reth_execution_types::BlockExecutionResult;
use reth_optimism_forks::OptimismHardforks;
use reth_primitives_traits::{
    Block, BlockBody, GotExpected, NodePrimitives, RecoveredBlock, SealedBlock, SealedHeader,
};

mod proof;
pub use proof::calculate_receipt_root_no_memo_optimism;

mod validation;
pub use validation::validate_block_post_execution;

/// Optimism consensus implementation.
///
/// Provides basic checks as outlined in the execution specs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpBeaconConsensus<ChainSpec> {
    chain_spec: Arc<ChainSpec>,
    max_extra_data_size: usize,
}

impl<ChainSpec> OpBeaconConsensus<ChainSpec> {
    /// Create a new instance of [`OpBeaconConsensus`].
    pub const fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self { chain_spec, max_extra_data_size: MAXIMUM_EXTRA_DATA_SIZE }
    }

    /// Returns the maximum allowed extra data size.
    pub const fn max_extra_data_size(&self) -> usize {
        self.max_extra_data_size
    }
}

impl<N, ChainSpec> FullConsensus<N> for OpBeaconConsensus<ChainSpec>
where
    N: NodePrimitives,
    ChainSpec: EthChainSpec<Header = N::BlockHeader> + OptimismHardforks + Debug + Send + Sync,
{
    fn validate_block_post_execution(
        &self,
        block: &RecoveredBlock<N::Block>,
        result: &BlockExecutionResult<N::Receipt>,
        receipt_root_bloom: Option<ReceiptRootBloom>,
        _block_access_list_hash: Option<alloy_primitives::B256>,
    ) -> Result<(), ConsensusError> {
        validate_block_post_execution(
            block.header(),
            self.chain_spec.as_ref(),
            result,
            receipt_root_bloom,
        )
    }
}

impl<B, ChainSpec> Consensus<B> for OpBeaconConsensus<ChainSpec>
where
    B: Block,
    B::Body: BlockBody,
    ChainSpec: EthChainSpec<Header = B::Header> + OptimismHardforks + Debug + Send + Sync,
{
    fn validate_body_against_header(
        &self,
        body: &B::Body,
        header: &SealedHeader<B::Header>,
    ) -> Result<(), ConsensusError> {
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

        Ok(())
    }

    fn validate_block_pre_execution(&self, block: &SealedBlock<B>) -> Result<(), ConsensusError> {
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

        if let Err(error) = block.ensure_transaction_root_valid() {
            return Err(ConsensusError::BodyTransactionRootDiff(error.into()))
        }

        if self.chain_spec.is_shanghai_active_at_timestamp(block.timestamp()) {
            validate_shanghai_withdrawals(block)?;
        }

        if self.chain_spec.is_cancun_active_at_timestamp(block.timestamp()) {
            validate_cancun_gas(block)?;
        }

        Ok(())
    }
}

impl<H, ChainSpec> HeaderValidator<H> for OpBeaconConsensus<ChainSpec>
where
    H: reth_primitives_traits::BlockHeader,
    ChainSpec: EthChainSpec<Header = H> + OptimismHardforks + Debug + Send + Sync,
{
    fn validate_header(&self, header: &SealedHeader<H>) -> Result<(), ConsensusError> {
        let header = header.header();

        if header.nonce() != Some(B64::ZERO) {
            return Err(ConsensusError::TheMergeNonceIsNotZero)
        }

        if header.ommers_hash() != EMPTY_OMMER_ROOT_HASH {
            return Err(ConsensusError::TheMergeOmmerRootIsNotEmpty)
        }

        validate_header_extra_data(header, self.max_extra_data_size)?;
        validate_header_gas(header)?;
        validate_header_base_fee(header, &self.chain_spec)
    }

    fn validate_header_against_parent(
        &self,
        header: &SealedHeader<H>,
        parent: &SealedHeader<H>,
    ) -> Result<(), ConsensusError> {
        validate_against_parent_hash_number(header.header(), parent)?;

        if self.chain_spec.is_bedrock_active_at_block(header.number()) {
            validate_against_parent_timestamp(header.header(), parent.header())?;
        }

        if self.chain_spec.is_wright_active_at_timestamp(header.timestamp()) {
            let base_fee = header.base_fee_per_gas().ok_or(ConsensusError::BaseFeeMissing)?;
            if base_fee != 0 {
                return Err(ConsensusError::BaseFeeDiff(GotExpected { expected: 0, got: base_fee }))
            }
        } else {
            validate_against_parent_eip1559_base_fee(
                header.header(),
                parent.header(),
                &self.chain_spec,
            )?;
        }

        if self.chain_spec.is_ecotone_active_at_timestamp(header.timestamp()) {
            let blob_gas_used = header.blob_gas_used().ok_or(ConsensusError::BlobGasUsedMissing)?;
            if blob_gas_used != 0 {
                return Err(ConsensusError::BlobGasUsedDiff(GotExpected {
                    got: blob_gas_used,
                    expected: 0,
                }))
            }

            let excess_blob_gas =
                header.excess_blob_gas().ok_or(ConsensusError::ExcessBlobGasMissing)?;
            if excess_blob_gas != 0 {
                return Err(ConsensusError::ExcessBlobGasDiff {
                    diff: GotExpected { got: excess_blob_gas, expected: 0 },
                    parent_excess_blob_gas: parent.excess_blob_gas().unwrap_or(0),
                    parent_blob_gas_used: parent.blob_gas_used().unwrap_or(0),
                })
            }
        }

        Ok(())
    }
}
