use super::{BlockHeader, Header};
use crate::InMemorySize;
use alloy_eips::BlockNumHash;
use alloy_primitives::{keccak256, BlockHash, Sealable};
#[cfg(any(test, feature = "test-utils"))]
use alloy_primitives::{BlockNumber, B256, U256};
use alloy_rlp::{Decodable, Encodable};
use bytes::BufMut;
use core::mem;
use derive_more::{AsRef, Deref};
use reth_codecs::add_arbitrary_tests;
use serde::{Deserialize, Serialize};
use crate::sync::OnceLock;

/// A [`Header`] that is sealed at a precalculated hash, use [`SealedHeader::unseal()`] if you want
/// to modify header.
#[derive(Debug, Clone, AsRef, Deref, Serialize, Deserialize)]
#[add_arbitrary_tests(rlp)]
pub struct SealedHeader<H = Header> {
    /// Locked Header hash, lazily computed if the header was created via
    /// [`SealedHeader::new_unhashed`].
    #[serde(skip)]
    hash: OnceLock<BlockHash>,
    /// Locked Header fields.
    #[as_ref]
    #[deref]
    header: H,
}

impl<H> SealedHeader<H> {
    /// Creates the sealed header with the corresponding block hash.
    #[inline]
    pub fn new(header: H, hash: BlockHash) -> Self {
        Self { header, hash: OnceLock::from(hash) }
    }

    /// Creates a sealed header without hashing the header up front. The hash will be computed
    /// lazily the first time it's requested.
    #[inline]
    pub fn new_unhashed(header: H) -> Self {
        Self { header, hash: OnceLock::new() }
    }

    /// Returns the sealed header fields.
    #[inline]
    pub const fn header(&self) -> &H {
        &self.header
    }

    /// Consumes the type and returns the wrapped header.
    #[inline]
    pub fn into_header(self) -> H {
        self.header
    }

    /// Extract raw header that can be modified.
    #[inline]
    pub fn unseal(self) -> H {
        self.header
    }
}

impl<H: Sealable> SealedHeader<H> {
    /// Hashes the header and creates a sealed header from it and its hash.
    pub fn seal_slow(header: H) -> Self {
        let hash = header.hash_slow();
        Self::new(header, hash)
    }

    /// Returns the block hash, computing and caching it if not already available.
    #[inline]
    pub fn hash(&self) -> BlockHash {
        *self.hash.get_or_init(|| self.header.hash_slow())
    }

    /// This is the inverse of [`SealedHeader::seal_slow`] which returns the raw header and hash.
    pub fn split(self) -> (H, BlockHash) {
        let hash = self.hash();
        (self.header, hash)
    }
}

impl<H: BlockHeader + Sealable> SealedHeader<H> {
    /// Return the number hash tuple.
    pub fn num_hash(&self) -> BlockNumHash {
        BlockNumHash::new(self.header.number(), self.hash())
    }
}

impl<H: InMemorySize> InMemorySize for SealedHeader<H> {
    /// Calculates a heuristic for the in-memory size of the [`SealedHeader`].
    #[inline]
    fn size(&self) -> usize {
        self.header.size() + mem::size_of::<BlockHash>()
    }
}

impl<H: Sealable> PartialEq for SealedHeader<H> {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl<H: Sealable> Eq for SealedHeader<H> {}

impl<H: Sealable> core::hash::Hash for SealedHeader<H> {
    fn hash<Ha: core::hash::Hasher>(&self, state: &mut Ha) {
        self.hash().hash(state)
    }
}

impl<H: Default + Sealable> Default for SealedHeader<H> {
    fn default() -> Self {
        Self::seal_slow(H::default())
    }
}

impl Encodable for SealedHeader {
    fn encode(&self, out: &mut dyn BufMut) {
        self.header.encode(out);
    }
}

impl Decodable for SealedHeader {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let b = &mut &**buf;
        let started_len = buf.len();

        // decode the header from temp buffer
        let header = Header::decode(b)?;

        // hash the consumed bytes, the rlp encoded header
        let consumed = started_len - b.len();
        let hash = keccak256(&buf[..consumed]);

        // update original buffer
        *buf = *b;

        Ok(Self::new(header, hash))
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl<H: super::test_utils::TestHeader> SealedHeader<H> {
    /// Returns a mutable reference to the header.
    pub fn header_mut(&mut self) -> &mut H {
        &mut self.header
    }

    /// Updates the block header.
    pub fn set_header(&mut self, header: H) {
        self.header = header
    }

    /// Updates the block hash.
    pub fn set_hash(&mut self, hash: BlockHash) {
        self.hash = OnceLock::from(hash)
    }

    /// Updates the parent block hash.
    pub fn set_parent_hash(&mut self, hash: BlockHash) {
        self.header.set_parent_hash(hash)
    }

    /// Updates the block number.
    pub fn set_block_number(&mut self, number: BlockNumber) {
        self.header.set_block_number(number);
    }

    /// Updates the block timestamp.
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.header.set_timestamp(timestamp);
    }

    /// Updates the block state root.
    pub fn set_state_root(&mut self, state_root: B256) {
        self.header.set_state_root(state_root);
    }

    /// Updates the block difficulty.
    pub fn set_difficulty(&mut self, difficulty: U256) {
        self.header.set_difficulty(difficulty);
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl<'a> arbitrary::Arbitrary<'a> for SealedHeader {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let header = Header::arbitrary(u)?;

        let sealed = header.seal_slow();
        let (header, seal) = sealed.into_parts();
        Ok(Self::new(header, seal))
    }
}

/// Bincode-compatible [`SealedHeader`] serde implementation.
#[cfg(feature = "serde-bincode-compat")]
pub(super) mod serde_bincode_compat {
    use alloy_consensus::serde_bincode_compat::Header;
    use alloy_primitives::BlockHash;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_with::{DeserializeAs, SerializeAs};

    /// Bincode-compatible [`super::SealedHeader`] serde implementation.
    ///
    /// Intended to use with the [`serde_with::serde_as`] macro in the following way:
    /// ```rust
    /// use reth_primitives_traits::{serde_bincode_compat, SealedHeader};
    /// use serde::{Deserialize, Serialize};
    /// use serde_with::serde_as;
    ///
    /// #[serde_as]
    /// #[derive(Serialize, Deserialize)]
    /// struct Data {
    ///     #[serde_as(as = "serde_bincode_compat::SealedHeader")]
    ///     header: SealedHeader,
    /// }
    /// ```
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SealedHeader<'a> {
        hash: BlockHash,
        header: Header<'a>,
    }

    impl<'a> From<&'a super::SealedHeader> for SealedHeader<'a> {
        fn from(value: &'a super::SealedHeader) -> Self {
            Self { hash: value.hash(), header: Header::from(&value.header) }
        }
    }

    impl<'a> From<SealedHeader<'a>> for super::SealedHeader {
        fn from(value: SealedHeader<'a>) -> Self {
            super::SealedHeader::new(value.header.into(), value.hash)
        }
    }

    impl SerializeAs<super::SealedHeader> for SealedHeader<'_> {
        fn serialize_as<S>(source: &super::SealedHeader, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            SealedHeader::from(source).serialize(serializer)
        }
    }

    impl<'de> DeserializeAs<'de, super::SealedHeader> for SealedHeader<'de> {
        fn deserialize_as<D>(deserializer: D) -> Result<super::SealedHeader, D::Error>
        where
            D: Deserializer<'de>,
        {
            SealedHeader::deserialize(deserializer).map(Into::into)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::super::{serde_bincode_compat, SealedHeader};

        use arbitrary::Arbitrary;
        use rand::Rng;
        use reth_testing_utils::generators;
        use serde::{Deserialize, Serialize};
        use serde_with::serde_as;

        #[test]
        fn test_sealed_header_bincode_roundtrip() {
            #[serde_as]
            #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
            struct Data {
                #[serde_as(as = "serde_bincode_compat::SealedHeader")]
                transaction: SealedHeader,
            }

            let mut bytes = [0u8; 1024];
            generators::rng().fill(bytes.as_mut_slice());
            let data = Data {
                transaction: SealedHeader::arbitrary(&mut arbitrary::Unstructured::new(&bytes))
                    .unwrap(),
            };

            let encoded = bincode::serialize(&data).unwrap();
            let decoded: Data = bincode::deserialize(&encoded).unwrap();
            assert_eq!(decoded, data);
        }
    }
}
