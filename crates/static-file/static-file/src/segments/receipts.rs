use crate::segments::Segment;
use alloy_primitives::BlockNumber;
use reth_codecs::Compact;
use reth_db_api::{cursor::DbCursorRO, table::{Table, Value}, tables, transaction::DbTx};
use reth_primitives_traits::NodePrimitives;
use reth_provider::{BlockReader, DBProvider, StaticFileProviderFactory};
use reth_static_file_types::StaticFileSegment;
use reth_storage_errors::provider::{ProviderError, ProviderResult};
use std::ops::RangeInclusive;

/// Static File segment responsible for [`StaticFileSegment::Receipts`] part of data.
#[derive(Debug, Default)]
pub struct Receipts;

impl<Provider> Segment<Provider> for Receipts
where
    Provider: StaticFileProviderFactory<Primitives: NodePrimitives<Receipt: Value + Compact>>
        + DBProvider
        + BlockReader,
    <tables::Receipts as Table>::Value: Into<<Provider::Primitives as NodePrimitives>::Receipt>,
{
    fn segment(&self) -> StaticFileSegment {
        StaticFileSegment::Receipts
    }

    fn copy_to_static_files(
        &self,
        provider: Provider,
        block_range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<()> {
        let mut static_file_writer =
            provider.get_static_file_writer(*block_range.start(), StaticFileSegment::Receipts)?;

        for block in block_range {
            static_file_writer.increment_block(block)?;

            let block_body_indices = provider
                .block_body_indices(block)?
                .ok_or(ProviderError::BlockBodyIndicesNotFound(block))?;

            let mut receipts_cursor = provider.tx_ref().cursor_read::<tables::Receipts>()?;
            let receipts_walker = receipts_cursor.walk_range(block_body_indices.tx_num_range())?;

            for receipt_entry in receipts_walker {
                let (tx_num, receipt) = receipt_entry.map_err(ProviderError::from)?;
                let receipt: <Provider::Primitives as NodePrimitives>::Receipt = receipt.into();
                static_file_writer.append_receipt(tx_num, &receipt)?;
            }
        }

        Ok(())
    }
}
