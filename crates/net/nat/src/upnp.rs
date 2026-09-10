//! UPnP/IGD port mapping (geth-style).
//!
//! Tries the preferred external port first without deleting foreign mappings. On conflict,
//! requests an alternative external port and returns that for ENR/enode announcement.
//!
//! Lease refresh runs until [`UpnpMappingGuard`] is dropped or [`UpnpMappingGuard::shutdown`] is
//! awaited; on stop the control point issues `DeletePortMapping` (WANIPConnection) so the IGD
//! frees the ports immediately instead of waiting for lease expiry.

use crate::NatEndpoint;
use igd_next::{
    aio::{
        tokio::{search_gateway, Tokio},
        Gateway,
    },
    PortMappingProtocol, SearchOptions,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};
use tokio::sync::oneshot;
use tracing::{debug, info, warn};

/// Default UPnP lease duration (seconds). Matches geth (~10 minutes).
pub const DEFAULT_LEASE_SECS: u32 = 600;

/// How long to wait for SSDP gateway discovery.
///
/// Measured LAN IGD response on this site is typically ~0.3–2s; igd-next's own default is 10s.
/// Keep 10s so slow/contended multicast (e.g. containers) still has headroom — our previous 3s
/// was too tight and showed up as `No response within timeout` in the archive CT.
const SEARCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Upper bound for a single `DeletePortMapping` SOAP call during shutdown.
const DELETE_TIMEOUT: Duration = Duration::from_secs(5);

/// How long [`UpnpMappingGuard::shutdown`] waits for the refresh task to finish deletes.
/// Covers TCP+UDP (two SOAP calls) plus a little slack.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(12);

const TCP_DESC: &str = "reth ethereum p2p";
const UDP_DESC: &str = "reth ethereum discovery";

/// Errors while mapping ports through an IGD.
#[derive(Debug, thiserror::Error)]
pub enum UpnpMapError {
    /// No IGD discovered on the LAN.
    #[error("UPnP gateway search failed: {0}")]
    Search(#[from] igd_next::SearchError),
    /// Could not read the gateway's WAN IP.
    #[error("failed to get external IP from IGD: {0}")]
    ExternalIp(String),
    /// TCP mapping failed.
    #[error("TCP port mapping failed: {0}")]
    Tcp(String),
    /// UDP mapping failed.
    #[error("UDP port mapping failed: {0}")]
    Udp(String),
    /// Could not determine a non-unspecified LAN address for the mapping.
    #[error("could not resolve local LAN address for UPnP mapping")]
    LocalAddr,
}

/// Discover IGD, map TCP+UDP without hijacking existing entries, return the dialable endpoint.
///
/// Prefer `preferred_port` as the external port for both protocols. If that external port is
/// already taken, fall back to an alternative external port (`AddAnyPortMapping` / random) —
/// **never** deletes another client's mapping first.
pub async fn map_ports(
    listen_tcp_port: u16,
    listen_udp_port: u16,
    preferred_port: u16,
) -> Result<(NatEndpoint, MappedGateway), UpnpMapError> {
    let gateway = search_gateway(SearchOptions {
        timeout: Some(SEARCH_TIMEOUT),
        single_search_timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    })
    .await?;

    let external_ip =
        gateway.get_external_ip().await.map_err(|e| UpnpMapError::ExternalIp(e.to_string()))?;

    let local_ip = local_ip_toward(gateway.addr).ok_or(UpnpMapError::LocalAddr)?;
    let tcp_local = SocketAddr::new(local_ip, listen_tcp_port);
    let udp_local = SocketAddr::new(local_ip, listen_udp_port);

    let tcp_ext = map_one(&gateway, PortMappingProtocol::TCP, preferred_port, tcp_local, TCP_DESC)
        .await
        .map_err(UpnpMapError::Tcp)?;
    let udp_ext = map_one(&gateway, PortMappingProtocol::UDP, preferred_port, udp_local, UDP_DESC)
        .await
        .map_err(UpnpMapError::Udp)?;

    let endpoint =
        NatEndpoint { ip: external_ip, tcp_port: tcp_ext, udp_port: udp_ext, via_upnp: true };

    if tcp_ext != preferred_port || udp_ext != preferred_port {
        info!(
            target: "net::nat",
            %external_ip,
            preferred_port,
            tcp_ext,
            udp_ext,
            local_ip = %local_ip,
            "NAT mapped alternative UPnP port(s)"
        );
    } else {
        info!(
            target: "net::nat",
            %external_ip,
            port = preferred_port,
            local_ip = %local_ip,
            "NAT mapped UPnP port"
        );
    }

    Ok((endpoint, MappedGateway { gateway, tcp_local, udp_local, endpoint }))
}

/// Keep-alive handle for periodic UPnP lease refresh / shutdown delete.
#[derive(Debug)]
pub struct MappedGateway {
    gateway: Gateway<Tokio>,
    tcp_local: SocketAddr,
    udp_local: SocketAddr,
    endpoint: NatEndpoint,
}

impl MappedGateway {
    /// Re-add the current TCP/UDP mappings (refresh lease). Does not delete foreign mappings.
    pub async fn refresh(&self) -> Result<(), UpnpMapError> {
        self.gateway
            .add_port(
                PortMappingProtocol::TCP,
                self.endpoint.tcp_port,
                self.tcp_local,
                DEFAULT_LEASE_SECS,
                TCP_DESC,
            )
            .await
            .map_err(|e| UpnpMapError::Tcp(e.to_string()))?;
        self.gateway
            .add_port(
                PortMappingProtocol::UDP,
                self.endpoint.udp_port,
                self.udp_local,
                DEFAULT_LEASE_SECS,
                UDP_DESC,
            )
            .await
            .map_err(|e| UpnpMapError::Udp(e.to_string()))?;
        debug!(
            target: "net::nat",
            tcp = self.endpoint.tcp_port,
            udp = self.endpoint.udp_port,
            "Refreshed UPnP port mappings"
        );
        Ok(())
    }

    /// Remove our TCP/UDP mappings via `DeletePortMapping` (best-effort both).
    ///
    /// Each SOAP call is capped by [`DELETE_TIMEOUT`] so a silent IGD cannot stall shutdown.
    pub async fn delete(&self) {
        match tokio::time::timeout(
            DELETE_TIMEOUT,
            self.gateway.remove_port(PortMappingProtocol::TCP, self.endpoint.tcp_port),
        )
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                warn!(
                    target: "net::nat",
                    tcp = self.endpoint.tcp_port,
                    %err,
                    "Failed to DeletePortMapping (TCP) on shutdown"
                );
            }
            Err(_) => {
                warn!(
                    target: "net::nat",
                    tcp = self.endpoint.tcp_port,
                    timeout_secs = DELETE_TIMEOUT.as_secs(),
                    "DeletePortMapping (TCP) timed out on shutdown"
                );
            }
        }
        match tokio::time::timeout(
            DELETE_TIMEOUT,
            self.gateway.remove_port(PortMappingProtocol::UDP, self.endpoint.udp_port),
        )
        .await
        {
            Ok(Ok(())) => {
                info!(
                    target: "net::nat",
                    tcp = self.endpoint.tcp_port,
                    udp = self.endpoint.udp_port,
                    "Deleted UPnP port mappings on shutdown"
                );
            }
            Ok(Err(err)) => {
                warn!(
                    target: "net::nat",
                    udp = self.endpoint.udp_port,
                    %err,
                    "Failed to DeletePortMapping (UDP) on shutdown"
                );
            }
            Err(_) => {
                warn!(
                    target: "net::nat",
                    udp = self.endpoint.udp_port,
                    timeout_secs = DELETE_TIMEOUT.as_secs(),
                    "DeletePortMapping (UDP) timed out on shutdown"
                );
            }
        }
    }

    /// The advertised endpoint established at map time.
    pub const fn endpoint(&self) -> NatEndpoint {
        self.endpoint
    }
}

/// UDP-only IGD lease (e.g. discv5 listen port ≠ discv4/RLPx UDP).
#[derive(Debug)]
struct MappedUdpPort {
    gateway: Gateway<Tokio>,
    local: SocketAddr,
    external: u16,
}

impl MappedUdpPort {
    async fn refresh(&self) -> Result<(), UpnpMapError> {
        self.gateway
            .add_port(
                PortMappingProtocol::UDP,
                self.external,
                self.local,
                DEFAULT_LEASE_SECS,
                UDP_DESC,
            )
            .await
            .map_err(|e| UpnpMapError::Udp(e.to_string()))?;
        debug!(target: "net::nat", udp = self.external, "Refreshed UPnP UDP port mapping");
        Ok(())
    }

    async fn delete(&self) {
        match tokio::time::timeout(
            DELETE_TIMEOUT,
            self.gateway.remove_port(PortMappingProtocol::UDP, self.external),
        )
        .await
        {
            Ok(Ok(())) => {
                info!(
                    target: "net::nat",
                    udp = self.external,
                    "Deleted UPnP UDP port mapping on shutdown"
                );
            }
            Ok(Err(err)) => {
                warn!(
                    target: "net::nat",
                    udp = self.external,
                    %err,
                    "Failed to DeletePortMapping (UDP) on shutdown"
                );
            }
            Err(_) => {
                warn!(
                    target: "net::nat",
                    udp = self.external,
                    timeout_secs = DELETE_TIMEOUT.as_secs(),
                    "DeletePortMapping (UDP) timed out on shutdown"
                );
            }
        }
    }
}

enum ActiveMapping {
    TcpUdp(MappedGateway),
    UdpOnly(MappedUdpPort),
}

impl ActiveMapping {
    async fn refresh(&self) -> Result<(), UpnpMapError> {
        match self {
            Self::TcpUdp(m) => m.refresh().await,
            Self::UdpOnly(m) => m.refresh().await,
        }
    }

    async fn delete(&self) {
        match self {
            Self::TcpUdp(m) => m.delete().await,
            Self::UdpOnly(m) => m.delete().await,
        }
    }
}

/// Owns the UPnP lease refresh task. Dropping (or awaiting [`Self::shutdown`]) stops refresh and
/// issues `DeletePortMapping` for the ports we opened — geth `Map` defer-delete semantics.
#[derive(Debug)]
#[must_use = "dropping UpnpMappingGuard deletes IGD port mappings"]
pub struct UpnpMappingGuard {
    /// `Some` until shutdown consumed / drop fired.
    shutdown_tx: Option<oneshot::Sender<oneshot::Sender<()>>>,
}

impl UpnpMappingGuard {
    /// Stop lease refresh, await `DeletePortMapping`, then return.
    ///
    /// Returns after at most [`SHUTDOWN_WAIT`] even if the IGD never answers — the node must not
    /// stall on a dead router. Drop path is fire-and-forget (no await).
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let (done_tx, done_rx) = oneshot::channel();
            if tx.send(done_tx).is_ok() {
                let _ = tokio::time::timeout(SHUTDOWN_WAIT, done_rx).await;
            }
        }
    }
}

impl Drop for UpnpMappingGuard {
    fn drop(&mut self) {
        // Best-effort: wake the refresh task so it can DeletePortMapping while the runtime lives.
        if let Some(tx) = self.shutdown_tx.take() {
            let (done_tx, _done_rx) = oneshot::channel();
            let _ = tx.send(done_tx);
        }
    }
}

fn spawn_active_mapping_refresh(mapped: ActiveMapping, interval: Duration) -> UpnpMappingGuard {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<oneshot::Sender<()>>();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // Skip the immediate first tick; mapping was just created.
        ticker.tick().await;
        tokio::pin!(shutdown_rx);
        loop {
            tokio::select! {
                done = &mut shutdown_rx => {
                    mapped.delete().await;
                    if let Ok(done_tx) = done {
                        let _ = done_tx.send(());
                    }
                    break;
                }
                _ = ticker.tick() => {
                    if let Err(err) = mapped.refresh().await {
                        warn!(target: "net::nat", %err, "Failed to refresh UPnP port mapping");
                    }
                }
            }
        }
    });
    UpnpMappingGuard { shutdown_tx: Some(shutdown_tx) }
}

/// Spawn a background task that refreshes UPnP leases until the returned guard is dropped /
/// shut down (then `DeletePortMapping`).
pub fn spawn_mapping_refresh(mapped: MappedGateway, interval: Duration) -> UpnpMappingGuard {
    spawn_active_mapping_refresh(ActiveMapping::TcpUdp(mapped), interval)
}

/// Map a single UDP listen port (e.g. discv5) without deleting foreign mappings.
///
/// Returns an [`UpnpMappingGuard`] that refreshes the lease and deletes the mapping on shutdown.
pub async fn map_udp_port(
    listen_udp_port: u16,
    preferred: u16,
) -> Result<(IpAddr, u16, UpnpMappingGuard), UpnpMapError> {
    let gateway = search_gateway(SearchOptions {
        timeout: Some(SEARCH_TIMEOUT),
        single_search_timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    })
    .await?;
    let external_ip =
        gateway.get_external_ip().await.map_err(|e| UpnpMapError::ExternalIp(e.to_string()))?;
    let local_ip = local_ip_toward(gateway.addr).ok_or(UpnpMapError::LocalAddr)?;
    let udp_local = SocketAddr::new(local_ip, listen_udp_port);
    let udp_ext = map_one(&gateway, PortMappingProtocol::UDP, preferred, udp_local, UDP_DESC)
        .await
        .map_err(UpnpMapError::Udp)?;
    info!(
        target: "net::nat",
        %external_ip,
        listen_udp_port,
        udp_ext,
        "NAT mapped additional UPnP UDP port"
    );
    let guard = spawn_active_mapping_refresh(
        ActiveMapping::UdpOnly(MappedUdpPort { gateway, local: udp_local, external: udp_ext }),
        Duration::from_secs(8 * 60),
    );
    Ok((external_ip, udp_ext, guard))
}

async fn map_one(
    gateway: &Gateway<Tokio>,
    protocol: PortMappingProtocol,
    preferred: u16,
    local: SocketAddr,
    desc: &str,
) -> Result<u16, String> {
    match gateway.add_port(protocol, preferred, local, DEFAULT_LEASE_SECS, desc).await {
        Ok(()) => Ok(preferred),
        Err(err) => {
            debug!(
                target: "net::nat",
                %protocol,
                preferred,
                %err,
                "Preferred UPnP external port unavailable; requesting alternative"
            );
            gateway
                .add_any_port(protocol, local, DEFAULT_LEASE_SECS, desc)
                .await
                .map_err(|e| e.to_string())
        }
    }
}

/// Best-effort LAN IP that can reach the gateway (for IGD `NewInternalClient`).
fn local_ip_toward(gateway: SocketAddr) -> Option<IpAddr> {
    let bind = match gateway {
        SocketAddr::V4(_) => SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)),
        SocketAddr::V6(_) => SocketAddr::from((std::net::Ipv6Addr::UNSPECIFIED, 0)),
    };
    let sock = UdpSocket::bind(bind).ok()?;
    sock.connect(gateway).ok()?;
    let ip = sock.local_addr().ok()?.ip();
    if ip.is_unspecified() || ip.is_loopback() {
        return None;
    }
    Some(ip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_ip_toward_loopback_gateway_is_none_or_some() {
        // Connecting to an unused high port on localhost may still yield a local address;
        // we only assert the helper does not panic.
        let _ = local_ip_toward(SocketAddr::from((Ipv4Addr::LOCALHOST, 9)));
    }
}
