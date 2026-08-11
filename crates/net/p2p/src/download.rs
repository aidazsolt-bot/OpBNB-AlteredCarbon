use reth_network_peers::PeerId;
use std::fmt::Debug;

/// Generic download client for peer penalization
#[auto_impl::auto_impl(&, Arc, Box)]
pub trait DownloadClient: Send + Sync + Debug {
    /// Penalize the peer for responding with a message
    /// that violates validation rules
    fn report_bad_message(&self, peer_id: PeerId);

    /// Returns how many peers the network is currently connected to.
    fn num_connected_peers(&self) -> usize;

    /// Highest advertised best-block number among connected peers.
    ///
    /// Used by reverse header sync to cap the *working* tip to what peers can actually serve
    /// (Status `best`), without dropping lagging peers. Returns `None` when unknown / no peers.
    fn max_peer_best_number(&self) -> Option<u64> {
        None
    }
}
