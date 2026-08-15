//! Helpers for resolving the external IP and optional UPnP port mapping.
//!
//! ## Feature Flags
//!
//! - `serde` (default): Enable serde support

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod net_if;
pub mod upnp;

pub use net_if::{NetInterfaceError, DEFAULT_NET_IF_NAME};
pub use upnp::{
    map_ports, map_udp_port, spawn_mapping_refresh, MappedGateway, UpnpMapError, DEFAULT_LEASE_SECS,
};

use std::{
    fmt,
    future::{poll_fn, Future},
    net::{AddrParseError, IpAddr, ToSocketAddrs},
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
    time::Duration,
};
use tracing::{debug, info, warn};

use crate::net_if::resolve_net_if_ip;
#[cfg(feature = "serde")]
use serde_with::{DeserializeFromStr, SerializeDisplay};

/// URLs to `GET` the external IP address.
///
/// Taken from: <https://stackoverflow.com/questions/3253701/get-public-external-ip-address>
const EXTERNAL_IP_APIS: &[&str] =
    &["https://ipinfo.io/ip", "https://icanhazip.com", "https://ifconfig.me"];

/// Dialable endpoint announced to peers after NAT resolution / UPnP mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NatEndpoint {
    /// Public (or advertised) IP.
    pub ip: IpAddr,
    /// TCP port peers should dial (may differ from the local listen port after UPnP remap).
    pub tcp_port: u16,
    /// UDP discovery port peers should use.
    pub udp_port: u16,
    /// `true` if established via IGD port mapping.
    pub via_upnp: bool,
}

impl NatEndpoint {
    /// Endpoint that reuses the local listen ports with a resolved public IP (HTTP / netif /
    /// fixed).
    pub const fn with_listen_ports(ip: IpAddr, tcp_port: u16, udp_port: u16) -> Self {
        Self { ip, tcp_port, udp_port, via_upnp: false }
    }
}

impl fmt::Display for NatEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (tcp={}, udp={})", self.ip, self.tcp_port, self.udp_port)
    }
}

/// All builtin resolvers.
#[derive(Debug, Clone, Eq, PartialEq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(SerializeDisplay, DeserializeFromStr))]
pub enum NatResolver {
    /// Resolve with any available resolver.
    ///
    /// Prefer UPnP/IGD port mapping (geth-style); fall back to HTTP public-IP lookup without
    /// mapping if no gateway is available.
    #[default]
    Any,
    /// Resolve external IP via `UPnP` and map listen ports through the IGD.
    Upnp,
    /// Resolve external IP via a network request (no port mapping).
    PublicIp,
    /// Use the given [`IpAddr`]
    ExternalIp(IpAddr),
    /// Use the given domain name as the external address to expose to peers.
    /// This is behaving essentially the same as [`NatResolver::ExternalIp`], but supports domain
    /// names. Domain names are resolved to IP addresses using the OS's resolver. The first IP
    /// address found is used.
    /// This may be useful in docker bridge networks where containers are usually queried by DNS
    /// instead of direct IP addresses.
    /// Note: the domain shouldn't include a port number. Only the IP address is resolved.
    ExternalAddr(String),
    /// Resolve external IP via the network interface.
    NetIf,
    /// Resolve nothing
    None,
}

impl NatResolver {
    /// Attempts to produce an IP address (best effort).
    pub async fn external_addr(self) -> Option<IpAddr> {
        external_addr_with(self).await
    }

    /// Returns the fixed ip, if it is [`NatResolver::ExternalIp`] or [`NatResolver::ExternalAddr`].
    ///
    /// In the case of [`NatResolver::ExternalAddr`], it will return the first IP address found for
    /// the domain.
    pub fn as_external_ip(self, port: u16) -> Option<IpAddr> {
        match self {
            Self::ExternalIp(ip) => Some(ip),
            Self::ExternalAddr(domain) => format!("{domain}:{port}")
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next().map(|addr| addr.ip())),
            _ => None,
        }
    }

    /// Whether this resolver should attempt UPnP/IGD port mapping.
    pub const fn wants_upnp_mapping(&self) -> bool {
        matches!(self, Self::Any | Self::Upnp)
    }
}

impl fmt::Display for NatResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("any"),
            Self::Upnp => f.write_str("upnp"),
            Self::PublicIp => f.write_str("publicip"),
            Self::ExternalIp(ip) => write!(f, "extip:{ip}"),
            Self::ExternalAddr(domain) => write!(f, "extaddr:{domain}"),
            Self::NetIf => f.write_str("netif"),
            Self::None => f.write_str("none"),
        }
    }
}

/// Error when parsing a [`NatResolver`]
#[derive(Debug, thiserror::Error)]
pub enum ParseNatResolverError {
    /// Failed to parse provided IP
    #[error(transparent)]
    AddrParseError(#[from] AddrParseError),
    /// Failed to parse due to unknown variant
    #[error("Unknown Nat Resolver variant: {0}")]
    UnknownVariant(String),
}

impl FromStr for NatResolver {
    type Err = ParseNatResolverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = match s {
            "any" => Self::Any,
            "upnp" => Self::Upnp,
            "none" => Self::None,
            "publicip" | "public-ip" => Self::PublicIp,
            "netif" => Self::NetIf,
            s => {
                if let Some(ip) = s.strip_prefix("extip:") {
                    Self::ExternalIp(ip.parse()?)
                } else if let Some(domain) = s.strip_prefix("extaddr:") {
                    Self::ExternalAddr(domain.to_string())
                } else {
                    return Err(ParseNatResolverError::UnknownVariant(format!(
                        "Unknown Nat Resolver: {s}"
                    )));
                }
            }
        };
        Ok(r)
    }
}

/// With this type you can resolve the external public IP address on an interval basis.
#[must_use = "Does nothing unless polled"]
pub struct ResolveNatInterval {
    resolver: NatResolver,
    future: Option<Pin<Box<dyn Future<Output = Option<IpAddr>> + Send>>>,
    interval: tokio::time::Interval,
}

impl fmt::Debug for ResolveNatInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolveNatInterval")
            .field("resolver", &self.resolver)
            .field("future", &self.future.as_ref().map(drop))
            .field("interval", &self.interval)
            .finish()
    }
}

impl ResolveNatInterval {
    fn with_interval(resolver: NatResolver, interval: tokio::time::Interval) -> Self {
        Self { resolver, future: None, interval }
    }

    /// Creates a new [`ResolveNatInterval`] that attempts to resolve the public IP with interval of
    /// period. See also [`tokio::time::interval`]
    #[track_caller]
    pub fn interval(resolver: NatResolver, period: Duration) -> Self {
        let interval = tokio::time::interval(period);
        Self::with_interval(resolver, interval)
    }

    /// Creates a new [`ResolveNatInterval`] that attempts to resolve the public IP with interval of
    /// period with the first attempt starting at `start`. See also [`tokio::time::interval_at`]
    #[track_caller]
    pub fn interval_at(
        resolver: NatResolver,
        start: tokio::time::Instant,
        period: Duration,
    ) -> Self {
        let interval = tokio::time::interval_at(start, period);
        Self::with_interval(resolver, interval)
    }

    /// Returns the resolver used by this interval
    pub const fn resolver(&self) -> &NatResolver {
        &self.resolver
    }

    /// Completes when the next [`IpAddr`] in the interval has been reached.
    pub async fn tick(&mut self) -> Option<IpAddr> {
        poll_fn(|cx| self.poll_tick(cx)).await
    }

    /// Polls for the next resolved [`IpAddr`] in the interval to be reached.
    ///
    /// This method can return the following values:
    ///
    ///  * `Poll::Pending` if the next [`IpAddr`] has not yet been resolved.
    ///  * `Poll::Ready(Option<IpAddr>)` if the next [`IpAddr`] has been resolved. This returns
    ///    `None` if the attempt was unsuccessful.
    pub fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Option<IpAddr>> {
        if self.interval.poll_tick(cx).is_ready() {
            self.future = Some(Box::pin(self.resolver.clone().external_addr()));
        }

        if let Some(mut fut) = self.future.take() {
            match fut.as_mut().poll(cx) {
                Poll::Ready(ip) => return Poll::Ready(ip),
                Poll::Pending => self.future = Some(fut),
            }
        }

        Poll::Pending
    }
}

/// Attempts to produce an IP address with all builtin resolvers (best effort).
pub async fn external_ip() -> Option<IpAddr> {
    external_addr_with(NatResolver::Any).await
}

/// Given a [`NatResolver`] attempts to produce an IP address (best effort).
///
/// Note: [`NatResolver::Any`] / [`NatResolver::Upnp`] use HTTP public-IP lookup here for the
/// periodic discv4 refresh path. Full UPnP **port mapping** is handled by
/// [`resolve_nat_endpoint`].
pub async fn external_addr_with(resolver: NatResolver) -> Option<IpAddr> {
    match resolver {
        NatResolver::Any | NatResolver::Upnp | NatResolver::PublicIp => resolve_external_ip().await,
        NatResolver::ExternalIp(ip) => Some(ip),
        NatResolver::NetIf => resolve_net_if_ip(DEFAULT_NET_IF_NAME)
            .inspect_err(|err| {
                debug!(target: "net::nat",
                     %err,
                    "Failed to resolve network interface IP"
                );
            })
            .ok(),
        NatResolver::ExternalAddr(domain) => tokio::net::lookup_host(format!("{domain}:0"))
            .await
            .inspect_err(|err| {
                debug!(target: "net::nat", %err, %domain, "Failed to resolve external address");
            })
            .ok()
            .and_then(|mut addrs| addrs.next().map(|addr| addr.ip())),
        NatResolver::None => None,
    }
}

/// Resolve the dialable NAT endpoint for the given listen ports.
///
/// * [`NatResolver::Any`]: UPnP/IGD mapping first (no hijack); on failure HTTP IP + listen ports.
/// * [`NatResolver::Upnp`]: UPnP only.
/// * [`NatResolver::PublicIp`] / `NetIf` / `External*`: IP resolution without mapping.
/// * [`NatResolver::None`]: `None`.
///
/// `listen_ip` is the RLPx bind address (`--addr`). Per FLOW-N01 / P2P-006, when a concrete
/// family is selected (including unspecified `0.0.0.0` / `::`), the announced IP must be the
/// **same family** — not a global HTTP preference for IPv4. Dual-stack without `--addr` is a
/// separate bind/announce path; this filter only enforces family consistency with the listen
/// socket that was actually opened.
pub async fn resolve_nat_endpoint(
    resolver: NatResolver,
    listen_tcp_port: u16,
    listen_udp_port: u16,
    listen_ip: IpAddr,
) -> Option<NatEndpoint> {
    let preferred = listen_tcp_port;
    let want_ipv4 = listen_ip.is_ipv4();

    if resolver.wants_upnp_mapping() {
        match map_ports(listen_tcp_port, listen_udp_port, preferred).await {
            Ok((endpoint, mapped)) => {
                if endpoint.ip.is_ipv4() != want_ipv4 {
                    warn!(
                        target: "net::nat",
                        listen_ip = %listen_ip,
                        mapped_ip = %endpoint.ip,
                        "UPnP mapped IP family does not match --addr listen family; ignoring mapping"
                    );
                } else {
                    // Refresh leases before they expire (geth uses ~8 min with 10 min lease).
                    spawn_mapping_refresh(mapped, Duration::from_secs(8 * 60));
                    return Some(endpoint);
                }
            }
            Err(err) => {
                if matches!(resolver, NatResolver::Upnp) {
                    warn!(target: "net::nat", %err, "UPnP NAT mapping failed");
                    return None;
                }
                warn!(
                    target: "net::nat",
                    %err,
                    "UPnP NAT mapping failed; falling back to HTTP public IP without port mapping"
                );
            }
        }
    }

    match resolver {
        NatResolver::None => None,
        NatResolver::Upnp => None,
        NatResolver::Any | NatResolver::PublicIp => {
            let ip = resolve_external_ip_matching_family(want_ipv4).await?;
            info!(
                target: "net::nat",
                %ip,
                listen_ip = %listen_ip,
                listen_tcp_port,
                listen_udp_port,
                "Resolved public IP via HTTP (no UPnP port mapping)"
            );
            Some(NatEndpoint::with_listen_ports(ip, listen_tcp_port, listen_udp_port))
        }
        NatResolver::ExternalIp(ip) => {
            if ip.is_ipv4() != want_ipv4 {
                warn!(
                    target: "net::nat",
                    listen_ip = %listen_ip,
                    %ip,
                    "extip family does not match --addr listen family; not announcing"
                );
                return None;
            }
            Some(NatEndpoint::with_listen_ports(ip, listen_tcp_port, listen_udp_port))
        }
        NatResolver::ExternalAddr(domain) => {
            let ip = tokio::net::lookup_host(format!("{domain}:0"))
                .await
                .ok()
                .and_then(|addrs| addrs.map(|a| a.ip()).find(|ip| ip.is_ipv4() == want_ipv4))?;
            Some(NatEndpoint::with_listen_ports(ip, listen_tcp_port, listen_udp_port))
        }
        NatResolver::NetIf => {
            let ip = resolve_net_if_ip(DEFAULT_NET_IF_NAME).ok()?;
            if ip.is_ipv4() != want_ipv4 {
                warn!(
                    target: "net::nat",
                    listen_ip = %listen_ip,
                    %ip,
                    "netif IP family does not match --addr listen family; not announcing"
                );
                return None;
            }
            Some(NatEndpoint::with_listen_ports(ip, listen_tcp_port, listen_udp_port))
        }
    }
}

/// HTTP public-IP lookup that accepts only addresses matching the listen socket family.
///
/// This is **not** an IPv4 preference: with `--addr ::` / IPv6 listen it requires IPv6 from the
/// APIs; with `--addr 0.0.0.0` it requires IPv4. First successful same-family response wins.
async fn resolve_external_ip_matching_family(want_ipv4: bool) -> Option<IpAddr> {
    let futures = EXTERNAL_IP_APIS
        .iter()
        .copied()
        .map(|url| resolve_external_ip_url_res_family(url, want_ipv4))
        .map(Box::pin);
    futures_util::future::select_ok(futures)
        .await
        .inspect_err(|err| {
            debug!(target: "net::nat",
            ?err,
                want_ipv4,
                external_ip_apis=?EXTERNAL_IP_APIS,
                "Failed to resolve same-family external IP from any API");
        })
        .ok()
        .map(|(ip, _)| ip)
}

async fn resolve_external_ip() -> Option<IpAddr> {
    let futures = EXTERNAL_IP_APIS.iter().copied().map(resolve_external_ip_url_res).map(Box::pin);
    futures_util::future::select_ok(futures)
        .await
        .inspect_err(|err| {
            debug!(target: "net::nat",
            ?err,
                external_ip_apis=?EXTERNAL_IP_APIS,
                "Failed to resolve external IP from any API");
        })
        .ok()
        .map(|(ip, _)| ip)
}

async fn resolve_external_ip_url_res_family(url: &str, want_ipv4: bool) -> Result<IpAddr, ()> {
    let ip = resolve_external_ip_url(url).await.ok_or(())?;
    if ip.is_ipv4() == want_ipv4 {
        Ok(ip)
    } else {
        debug!(
            target: "net::nat",
            %url,
            %ip,
            want_ipv4,
            "Ignoring HTTP public IP with family mismatch vs listen"
        );
        Err(())
    }
}

async fn resolve_external_ip_url_res(url: &str) -> Result<IpAddr, ()> {
    resolve_external_ip_url(url).await.ok_or(())
}

async fn resolve_external_ip_url(url: &str) -> Option<IpAddr> {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(10)).build().ok()?;
    let response = client.get(url).send().await.ok()?;
    let response = response.error_for_status().ok()?;
    let text = response.text().await.ok()?;
    text.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[tokio::test]
    #[ignore]
    async fn get_external_ip() {
        reth_tracing::init_test_tracing();
        let ip = external_ip().await;
        dbg!(ip);
    }

    #[tokio::test]
    #[ignore]
    async fn get_external_ip_interval() {
        reth_tracing::init_test_tracing();
        let mut interval = ResolveNatInterval::interval(Default::default(), Duration::from_secs(5));

        let ip = interval.tick().await;
        dbg!(ip);
        let ip = interval.tick().await;
        dbg!(ip);
    }

    #[test]
    fn as_external_ip_test() {
        let resolver = NatResolver::ExternalAddr("localhost".to_string());
        let ip = resolver.as_external_ip(30303).expect("localhost should be resolvable");

        if ip.is_ipv4() {
            assert_eq!(ip, IpAddr::V4(Ipv4Addr::LOCALHOST));
        } else {
            assert_eq!(ip, IpAddr::V6(Ipv6Addr::LOCALHOST));
        }
    }

    #[test]
    fn test_from_str() {
        assert_eq!(NatResolver::Any, "any".parse().unwrap());
        assert_eq!(NatResolver::None, "none".parse().unwrap());

        let ip = NatResolver::ExternalIp(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let s = "extip:0.0.0.0";
        assert_eq!(ip, s.parse().unwrap());
        assert_eq!(ip.to_string(), s);
    }

    #[test]
    fn wants_upnp() {
        assert!(NatResolver::Any.wants_upnp_mapping());
        assert!(NatResolver::Upnp.wants_upnp_mapping());
        assert!(!NatResolver::PublicIp.wants_upnp_mapping());
    }
}
