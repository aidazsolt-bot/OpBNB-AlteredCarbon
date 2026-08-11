//! opBNB milli-second timestamps encoded in `mixHash`.
//!
//! bnb-chain/op-geth stores the sub-second component in the first two bytes of
//! `mixHash` and validates `MilliTimestamp = Time*1000 + ms` instead of second
//! resolution (`Header.MilliTimestamp`). With Volta (500ms) / Fourier (250ms)
//! block times, consecutive blocks often share the same unix second while still
//! having strictly increasing milli-timestamps.

use alloy_consensus::BlockHeader;
use alloy_primitives::B256;
use reth_consensus::ConsensusError;

/// Milliseconds encoded in the first two bytes of `mix_hash` (big-endian).
///
/// Mirrors bnb-chain/op-geth `Header.millisecondes()`.
#[inline]
pub fn opbnb_milliseconds_from_mix_hash(mix_hash: B256) -> u64 {
    if mix_hash.is_zero() {
        return 0;
    }
    let bytes = mix_hash.as_slice();
    u16::from_be_bytes([bytes[0], bytes[1]]) as u64
}

/// Full milli-timestamp: `timestamp * 1000 + milliseconds(mix_hash)`.
///
/// Mirrors bnb-chain/op-geth `Header.MilliTimestamp()`.
#[inline]
pub fn opbnb_milli_timestamp(header: &impl BlockHeader) -> u64 {
    header
        .timestamp()
        .saturating_mul(1000)
        .saturating_add(opbnb_milliseconds_from_mix_hash(header.mix_hash().unwrap_or_default()))
}

/// Validates that the child header's milli-timestamp is strictly greater than the parent's.
///
/// Used on opBNB instead of second-resolution [`validate_against_parent_timestamp`].
#[inline]
pub fn validate_against_parent_opbnb_milli_timestamp<H: BlockHeader>(
    header: &H,
    parent: &H,
) -> Result<(), ConsensusError> {
    let parent_milli = opbnb_milli_timestamp(parent);
    let header_milli = opbnb_milli_timestamp(header);
    if header_milli <= parent_milli {
        return Err(ConsensusError::TimestampIsInPast {
            parent_timestamp: parent_milli,
            timestamp: header_milli,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::Header;
    use alloy_primitives::B256;

    fn header_with(ts: u64, mix: B256) -> Header {
        Header { timestamp: ts, mix_hash: mix, ..Default::default() }
    }

    #[test]
    fn milli_from_mix_hash_big_endian() {
        // Live opBNB block 173253771: mixHash starts 0x01f4 → 500ms
        let mut bytes = [0u8; 32];
        bytes[0] = 0x01;
        bytes[1] = 0xf4;
        assert_eq!(opbnb_milliseconds_from_mix_hash(B256::from(bytes)), 500);
    }

    #[test]
    fn equal_seconds_increasing_millis_ok() {
        // Blocks 173253771 / 173253772 both at unix second 1786430373,
        // with 500ms and 750ms respectively.
        let mut parent_mix = [0u8; 32];
        parent_mix[0] = 0x01;
        parent_mix[1] = 0xf4; // 500
        parent_mix[2] = 0x01;
        parent_mix[3] = 0x02;

        let mut child_mix = [0u8; 32];
        child_mix[0] = 0x02;
        child_mix[1] = 0xee; // 750
        child_mix[2] = 0x01;
        child_mix[3] = 0x02;

        let parent = header_with(1786430373, B256::from(parent_mix));
        let child = header_with(1786430373, B256::from(child_mix));

        assert_eq!(opbnb_milli_timestamp(&parent), 1786430373500);
        assert_eq!(opbnb_milli_timestamp(&child), 1786430373750);
        assert!(validate_against_parent_opbnb_milli_timestamp(&child, &parent).is_ok());
    }

    #[test]
    fn equal_milli_timestamp_rejected() {
        let mix = B256::from({
            let mut b = [0u8; 32];
            b[0] = 0x01;
            b[1] = 0xf4;
            b
        });
        let parent = header_with(100, mix);
        let child = header_with(100, mix);
        assert!(validate_against_parent_opbnb_milli_timestamp(&child, &parent).is_err());
    }
}
