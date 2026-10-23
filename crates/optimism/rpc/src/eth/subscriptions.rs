//! Stream subscription helpers for OP `eth_` pubsub.

use crate::{OpEthApi, OpEthApiError};
use reth_rpc_convert::RpcConvert;
use reth_rpc_eth_api::{helpers::EthSubscriptions, RpcNodeCore};

impl<N, Rpc> EthSubscriptions for OpEthApi<N, Rpc>
where
    N: RpcNodeCore,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = OpEthApiError>,
{
}
