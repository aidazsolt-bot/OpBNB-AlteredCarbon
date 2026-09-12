//! Standalone opBNB-shaped RLPx handshake probe: listen, complete a Spec handshake, then
//! disconnect the peer.
//!
//! Sequence (devp2p / eth Spec):
//! ```text
//! TCP accept
//!   → ECIES (auth)
//!   → p2p Hello (advertise eth/66–69 + optional snap/2)
//!   → eth Status (chain 204 / opBNB fork id)
//!   → p2p Disconnect(DisconnectRequested = 0x00)
//! ```
//!
//! This is **not** a Headers/Bodies/Snap server. After Status succeeds we immediately send a
//! voluntary disconnect so dialing clients can verify Hello/Status negotiation without staying
//! connected.
//!
//! Notes:
//! * `snap/1` is **not** advertised yet — wire only has `snap/2` in-tree (PORT-PIPE-015 /
//!   PORT-FLOW-SNAP-01). Enable `--snap` to offer `snap/2`.
//! * Do not extend `discv5` `manual_discovery`; that tool is UDP-only.
//!
//! Usage:
//! ```text
//! cargo run -p reth-eth-wire --example handshake_reject -- [OPTIONS]
//!
//! OPTIONS:
//!   --listen <addr>   TCP bind (default: 0.0.0.0:30304). Use with --serve.
//!   --serve           Accept inbound peers (default if no --dial / --dial-defaults).
//!   --dial <enode>    Outbound: dial this enode, handshake, DisconnectRequested.
//!   --dial-defaults   Outbound: dial the built-in opBNB trusted + official bootnodes.
//!   --snap            Advertise snap/2 in Hello
//!   --once            With --serve: exit after one inbound peer
//! ```

use alloy_primitives::U256;
use reth_chainspec::EthChainSpec;
use reth_ecies::stream::ECIESStream;
use reth_eth_wire::{
    CanDisconnect, DisconnectReason, EthNetworkPrimitives, EthVersion, HelloMessageWithProtocols,
    ProtocolVersion, UnauthedEthStream, UnauthedP2PStream, UnifiedStatus,
};
use reth_ethereum_forks::{Hardforks, Head};
use reth_network_peers::{pk2id, NodeRecord};
use reth_optimism_chainspec::OPBNB_MAINNET;
use secp256k1::{SecretKey, SECP256K1};
use std::{net::SocketAddr, str::FromStr, time::Duration};
use tokio::net::{TcpListener, TcpStream};

/// Same peers `manual_discovery` probes — RLPx TCP (devp2p port from the enode).
const DEFAULT_DIAL_TARGETS: &[&str] = &[
    "enode://a624fcf5276052da1f3a8151fd69e0e406903ff84d355301887d678f029160d02ab91583ed921812da04eefb12b24f4c71912c6f41603194b8c5ac8ef01ef421@167.235.95.170:30305",
    "enode://9967d43687535151322e6b5fc4f745e8df739e1cc26b70497083d3e68b31cd9bba8efc8d8cb9992e29a783872b785f4a023dc36260e7b29891e182e5238d1b7e@157.180.98.155:30315",
    "enode://547af4cbde12708f6d484f6182e9568da95404b0914fb4db4efa57794133427ca6968b86ca1322b29671d39e5db44e417f3be4b83a4c4d3772a708f82ff3948e@54.178.145.73:30303",
    "enode://50849e69a823e74db2bdda011cd85f7ccbebb53a0655008d4cb31f948877b36376489ff848efb8f5c5a523024ede177ee6c944d0c487c1a57b04b51f1a3d8923@54.227.72.206:30303",
];

struct Args {
    listen: SocketAddr,
    snap: bool,
    once: bool,
    serve: bool,
    dial: Vec<String>,
}

fn parse_args() -> Args {
    let mut listen = SocketAddr::from_str("0.0.0.0:30304").expect("default listen");
    let mut snap = false;
    let mut once = false;
    let mut serve = false;
    let mut dial = Vec::new();
    let mut dial_defaults = false;

    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--listen" => {
                let v = iter.next().expect("--listen requires host:port");
                listen = SocketAddr::from_str(&v).expect("invalid --listen");
            }
            "--snap" => snap = true,
            "--once" => once = true,
            "--serve" => serve = true,
            "--dial" => {
                let v = iter.next().expect("--dial requires an enode URL");
                dial.push(v);
            }
            "--dial-defaults" => dial_defaults = true,
            other => panic!("unknown argument: {other}"),
        }
    }

    if dial_defaults {
        dial.extend(DEFAULT_DIAL_TARGETS.iter().map(|s| (*s).to_string()));
    }
    // Listen-only when nothing else was requested.
    if !serve && dial.is_empty() {
        serve = true;
    }

    Args { listen, snap, once, serve, dial }
}

fn opbnb_head() -> Head {
    Head {
        number: 0,
        hash: OPBNB_MAINNET.genesis_hash(),
        difficulty: U256::ZERO,
        total_difficulty: U256::ZERO,
        timestamp: OPBNB_MAINNET.genesis_timestamp(),
    }
}

fn build_hello(id: reth_network_peers::PeerId, listen_port: u16, snap: bool) -> HelloMessageWithProtocols {
    let mut hello = HelloMessageWithProtocols {
        protocol_version: ProtocolVersion::V5,
        client_version: format!("{}/handshake-reject", reth_primitives_traits::constants::RETH_CLIENT_VERSION),
        // Highest shared eth version wins; include 66–69 so eth/68 (typical opBNB) and eth/69 both work.
        protocols: vec![
            EthVersion::Eth69.into(),
            EthVersion::Eth68.into(),
            EthVersion::Eth67.into(),
            EthVersion::Eth66.into(),
        ],
        port: listen_port,
        id,
    };
    if snap {
        hello = hello.with_snap(true);
    }
    hello
}

async fn complete_handshake<S>(
    stream: S,
    label: &str,
    secret_key: SecretKey,
    listen_port: u16,
    snap: bool,
) where
    S: futures::Stream<Item = Result<alloy_primitives::bytes::BytesMut, std::io::Error>>
        + futures::Sink<alloy_primitives::bytes::Bytes, Error = std::io::Error>
        + Unpin
        + Send
        + Sync,
{
    let local_id = pk2id(&secret_key.public_key(SECP256K1));
    let hello = build_hello(local_id, listen_port, snap);

    let (p2p_stream, their_hello) = match UnauthedP2PStream::new(stream).handshake(hello).await {
        Ok(ok) => ok,
        Err(err) => {
            println!("p2p Hello failed ({label}): {err}");
            return;
        }
    };

    println!(
        "Hello OK ({label}) peer_id={} client_version={} caps={:?}",
        their_hello.id, their_hello.client_version, their_hello.capabilities
    );
    println!("shared caps: {:?}", p2p_stream.shared_capabilities());

    let eth_version = match p2p_stream.shared_capabilities().eth_version() {
        Ok(v) => v,
        Err(err) => {
            println!("no shared eth capability ({err}); sending UselessPeer");
            let mut p2p = p2p_stream;
            let _ = p2p.disconnect(DisconnectReason::UselessPeer).await;
            return;
        }
    };

    let head = opbnb_head();
    let mut status = UnifiedStatus::spec_builder(&*OPBNB_MAINNET, &head);
    status.set_eth_version(eth_version);
    let fork_filter = OPBNB_MAINNET.fork_filter(head);

    println!(
        "sending eth Status version={eth_version:?} chain={} genesis={} forkid={:?}",
        status.chain, status.genesis, status.forkid
    );

    let (mut eth_stream, their_status) = match UnauthedEthStream::new(p2p_stream)
        .handshake::<EthNetworkPrimitives>(status, fork_filter)
        .await
    {
        Ok(ok) => ok,
        Err(err) => {
            println!("eth Status handshake failed ({label}): {err}");
            return;
        }
    };

    println!("Status OK ({label}) remote={}", their_status.into_message());

    match eth_stream.disconnect(DisconnectReason::DisconnectRequested).await {
        Ok(()) => println!("sent Disconnect(DisconnectRequested) → closed ({label})"),
        Err(err) => println!("disconnect ({label}) failed: {err}"),
    }
}

async fn handle_inbound(
    incoming: TcpStream,
    remote: SocketAddr,
    secret_key: SecretKey,
    listen_port: u16,
    snap: bool,
) {
    println!("--- inbound from {remote} ---");
    let stream = match ECIESStream::incoming(incoming, secret_key).await {
        Ok(s) => s,
        Err(err) => {
            println!("ECIES failed from {remote}: {err}");
            return;
        }
    };
    complete_handshake(stream, &remote.to_string(), secret_key, listen_port, snap).await;
}

async fn dial_peer(enode: &str, secret_key: SecretKey, listen_port: u16, snap: bool) {
    println!("--- dial {enode} ---");
    let record = match NodeRecord::from_str(enode) {
        Ok(r) => r,
        Err(err) => {
            println!("bad enode: {err}");
            return;
        }
    };
    let addr = SocketAddr::new(record.address, record.tcp_port);
    let tcp = match tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(addr)).await {
        Ok(Ok(s)) => s,
        Ok(Err(err)) => {
            println!("TCP connect {addr} failed: {err}");
            return;
        }
        Err(_) => {
            println!("TCP connect {addr} timed out");
            return;
        }
    };
    println!("TCP connected to {addr}");
    let stream = match ECIESStream::connect(tcp, secret_key, record.id).await {
        Ok(s) => s,
        Err(err) => {
            println!("ECIES connect to {addr} failed: {err}");
            return;
        }
    };
    complete_handshake(stream, &addr.to_string(), secret_key, listen_port, snap).await;
}

#[tokio::main]
async fn main() {
    reth_tracing::init_test_tracing();

    let args = parse_args();
    let mut sk_bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut sk_bytes);
    let secret_key = SecretKey::from_slice(&sk_bytes).expect("valid secret key");
    let local_id = pk2id(&secret_key.public_key(SECP256K1));

    println!("local peer id: {local_id}");
    println!(
        "Hello caps: eth/69,68,67,66{} → after Status: DisconnectRequested",
        if args.snap { " + snap/2" } else { "" }
    );

    for enode in &args.dial {
        dial_peer(enode, secret_key, args.listen.port(), args.snap).await;
    }

    if !args.serve {
        return;
    }

    let listener = TcpListener::bind(args.listen).await.expect("bind listen");
    let local_addr = listener.local_addr().expect("local_addr");
    println!("listening on {local_addr}");
    println!(
        "enode hint: enode://{local_id}@<reachable-host>:{}  (replace with this host's dialable IP)",
        local_addr.port()
    );
    println!("waiting for inbound (no discv4/5 announce — only explicit dials find us)…");

    loop {
        let (incoming, remote) = match listener.accept().await {
            Ok(ok) => ok,
            Err(err) => {
                println!("accept error: {err}");
                continue;
            }
        };
        handle_inbound(incoming, remote, secret_key, local_addr.port(), args.snap).await;
        if args.once {
            break;
        }
    }
}
