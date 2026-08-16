//! Local header seed for reverse-sync tip fetch.
//!
//! When the consensus engine buffers a NewPayload tip that the pipeline should sync to, that tip
//! header is often not yet (or never) available from the connected eth/66 peers. Seeding it here
//! lets [`HeadersClient`] serve the tip locally so [`ReverseHeadersDownloader`] can start walking
//! the parent chain without treating empty P2P tip responses as peer faults.

use crate::{
    bodies::client::BodiesClient,
    download::DownloadClient,
    headers::client::{HeadersClient, HeadersRequest},
    priority::Priority,
    BlockClient,
};
use alloy_eips::BlockHashOrNumber;
use alloy_primitives::B256;
use reth_eth_wire_types::HeadersDirection;
use reth_network_peers::{PeerId, WithPeerId};
use reth_primitives_traits::{Block, BlockHeader};
use std::{
    collections::HashMap,
    fmt,
    future::Future,
    ops::RangeInclusive,
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

/// Shared cache of headers known locally (e.g. from engine NewPayload).
#[derive(Debug)]
pub struct HeaderSeed<H> {
    by_hash: RwLock<HashMap<B256, H>>,
    by_number: RwLock<HashMap<u64, B256>>,
}

impl<H> Default for HeaderSeed<H> {
    fn default() -> Self {
        Self { by_hash: RwLock::new(HashMap::new()), by_number: RwLock::new(HashMap::new()) }
    }
}

impl<H: BlockHeader + Clone> HeaderSeed<H> {
    /// Inserts a header so subsequent tip fetches can be served locally.
    pub fn insert(&self, hash: B256, header: H) {
        let number = header.number();
        self.by_number.write().expect("header seed lock").insert(number, hash);
        self.by_hash.write().expect("header seed lock").insert(hash, header);
    }

    /// Returns a cloned header for `start` when the seed has a single-header match.
    fn get(&self, start: BlockHashOrNumber) -> Option<H> {
        match start {
            BlockHashOrNumber::Hash(hash) => self.by_hash.read().expect("header seed lock").get(&hash).cloned(),
            BlockHashOrNumber::Number(number) => {
                let hash = *self.by_number.read().expect("header seed lock").get(&number)?;
                self.by_hash.read().expect("header seed lock").get(&hash).cloned()
            }
        }
    }
}

/// [`BlockClient`] wrapper that answers single-header tip requests from [`HeaderSeed`] first.
#[derive(Debug)]
pub struct SeededBlockClient<C: BlockClient> {
    inner: C,
    seed: Arc<HeaderSeed<<C::Block as Block>::Header>>,
}

impl<C: BlockClient> Clone for SeededBlockClient<C> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), seed: Arc::clone(&self.seed) }
    }
}

impl<C: BlockClient> SeededBlockClient<C> {
    /// Wraps `inner` and serves tip headers from `seed` when present.
    pub const fn new(inner: C, seed: Arc<HeaderSeed<<C::Block as Block>::Header>>) -> Self {
        Self { inner, seed }
    }

    /// Shared seed handle for the consensus engine to insert NewPayload tips.
    pub fn seed(&self) -> Arc<HeaderSeed<<C::Block as Block>::Header>> {
        Arc::clone(&self.seed)
    }
}

impl<C: BlockClient> DownloadClient for SeededBlockClient<C> {
    fn report_bad_message(&self, peer_id: PeerId) {
        // Local seed responses use PeerId::ZERO — never penalize that sentinel.
        if peer_id.is_zero() {
            return;
        }
        self.inner.report_bad_message(peer_id);
    }

    fn num_connected_peers(&self) -> usize {
        self.inner.num_connected_peers()
    }

    fn max_peer_best_number(&self) -> Option<u64> {
        self.inner.max_peer_best_number()
    }
}

impl<C: BlockClient> BodiesClient for SeededBlockClient<C> {
    type Body = <C as BodiesClient>::Body;
    type Output = <C as BodiesClient>::Output;

    fn get_block_bodies_with_priority_and_range_hint(
        &self,
        hashes: Vec<B256>,
        priority: Priority,
        range_hint: Option<RangeInclusive<u64>>,
    ) -> Self::Output {
        self.inner.get_block_bodies_with_priority_and_range_hint(hashes, priority, range_hint)
    }
}

/// Ready future for a locally seeded single-header response.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct SeededHeadersFut<H> {
    header: Option<H>,
}

impl<H: Unpin> Future for SeededHeadersFut<H> {
    type Output = crate::error::PeerRequestResult<Vec<H>>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let header = self.header.take().expect("polled after completion");
        Poll::Ready(Ok(WithPeerId::new(PeerId::ZERO, vec![header])))
    }
}

/// Either a seeded ready response or the inner network future.
pub enum SeededOrInnerFut<C: HeadersClient> {
    /// Served from [`HeaderSeed`].
    Seeded(SeededHeadersFut<C::Header>),
    /// Forwarded to the inner client.
    Inner(C::Output),
}

impl<C: HeadersClient> fmt::Debug for SeededOrInnerFut<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seeded(_) => f.write_str("Seeded"),
            Self::Inner(_) => f.write_str("Inner"),
        }
    }
}

impl<C: HeadersClient> Future for SeededOrInnerFut<C> {
    type Output = crate::error::PeerRequestResult<Vec<C::Header>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: we never move the futures out of the enum.
        unsafe {
            match self.get_unchecked_mut() {
                Self::Seeded(fut) => Pin::new_unchecked(fut).poll(cx),
                Self::Inner(fut) => Pin::new_unchecked(fut).poll(cx),
            }
        }
    }
}

impl<C: BlockClient> HeadersClient for SeededBlockClient<C> {
    type Header = <C as HeadersClient>::Header;
    type Output = SeededOrInnerFut<C>;

    fn get_headers_with_priority(
        &self,
        request: HeadersRequest,
        priority: Priority,
    ) -> Self::Output {
        // Tip fetch is always limit=1. Serve from seed so reverse sync can start without peers that
        // know the FCU tip hash yet.
        if request.limit == 1 &&
            matches!(
                request.direction,
                HeadersDirection::Falling | HeadersDirection::Rising
            )
        {
            if let Some(header) = self.seed.get(request.start) {
                return SeededOrInnerFut::Seeded(SeededHeadersFut { header: Some(header) });
            }
        }

        SeededOrInnerFut::Inner(self.inner.get_headers_with_priority(request, priority))
    }
}

impl<C: BlockClient> BlockClient for SeededBlockClient<C> {
    type Block = C::Block;
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::Header;

    #[test]
    fn header_seed_serves_by_hash_and_number() {
        let seed = HeaderSeed::<Header>::default();
        let mut header = Header::default();
        header.number = 42;
        let hash = B256::repeat_byte(0xab);
        seed.insert(hash, header);

        assert_eq!(seed.get(hash.into()).unwrap().number, 42);
        assert_eq!(seed.get(42u64.into()).unwrap().number, 42);
        assert!(seed.get(B256::ZERO.into()).is_none());
    }
}
