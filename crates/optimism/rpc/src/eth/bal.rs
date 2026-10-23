//! Block access list helpers for OP `eth_` API.

use crate::{OpEthApi, OpEthApiError};
use reth_rpc_convert::RpcConvert;
use reth_rpc_eth_api::{helpers::bal::GetBlockAccessList, FromEvmError, RpcNodeCore};

impl<N, Rpc> GetBlockAccessList for OpEthApi<N, Rpc>
where
    N: RpcNodeCore,
    OpEthApiError: FromEvmError<N::Evm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = OpEthApiError, Evm = N::Evm>,
{
}
