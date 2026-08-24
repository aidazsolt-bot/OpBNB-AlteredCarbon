//! UPnP/IGD port mapping (geth-style).
//!
//! Tries the preferred external port first without deleting foreign mappings. On conflict,
//! requests an alternative external port and returns that for ENR/enode announcement.

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
use tracing::{debug, info, warn};

/// Default UPnP lease duration (seconds). Matches geth (~10 minutes).
pub const DEFAULT_LEASE_SECS: u32 = 600;

/// How long to wait for SSDP gateway discovery.
///
/// Measured LAN IGD response on this site is typically ~0.3–2s; igd-next's own default is 10s.
/// Keep 10s so slow/contended multicast (e.g. containers) still has headroom — our previous 3s
/// was too tight and showed up as `No response within timeout` in the archive CT.
const SEARCH_TIMEOUT: Duration = Duration::from_secs(10);

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
        .map_err(|e| UpnpMapError::Tcp(e))?;
    let udp_ext = map_one(&gateway, PortMappingProtocol::UDP, preferred_port, udp_local, UDP_DESC)
        .await
        .map_err(|e| UpnpMapError::Udp(e))?;

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

/// Keep-alive handle for periodic UPnP lease refresh.
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

    /// The advertised endpoint established at map time.
    pub const fn endpoint(&self) -> NatEndpoint {
        self.endpoint
    }
}

/// Map a single UDP listen port (e.g. discv5) without deleting foreign mappings.
pub async fn map_udp_port(
    listen_udp_port: u16,
    preferred: u16,
) -> Result<(IpAddr, u16), UpnpMapError> {
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
    Ok((external_ip, udp_ext))
}

/// Spawn a background task that refreshes UPnP leases until the process exits.
pub fn spawn_mapping_refresh(mapped: MappedGateway, interval: Duration) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // Skip the immediate first tick; mapping was just created.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if let Err(err) = mapped.refresh().await {
                warn!(target: "net::nat", %err, "Failed to refresh UPnP port mapping");
            }
        }
    });
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
