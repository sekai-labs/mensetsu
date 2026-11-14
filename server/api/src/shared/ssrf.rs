use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use reqwest::{redirect::Policy, Url};

use crate::shared::error::AppError;

/// Returns true if the IP address belongs to private, loopback, link-local,
/// cloud metadata, or other non-public ranges.
pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 169 && octets[1] == 254) // Link-local & cloud metadata (169.254.169.254)
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
                || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
                || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
                || octets[0] >= 240
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_multicast()
                || (segments[0] & 0xffc0) == 0xfec0
                || (segments[0] == 0x2001 && segments[1] == 0x0db8)
                || ip
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| is_blocked_ip(IpAddr::V4(mapped)))
        }
    }
}

/// Resolves a URL, verifies that its resolved IP addresses are safe from SSRF
/// (or explicitly allowlisted for self-hosted private deployments), and returns
/// the parsed URL and pinned SocketAddr list.
pub async fn resolve_and_verify_url(
    raw_url: &str,
    require_https: bool,
    allowlist: &[String],
) -> Result<(Url, Vec<SocketAddr>), AppError> {
    let url = Url::parse(raw_url)
        .map_err(|_| AppError::Validation(format!("Invalid URL: '{raw_url}'")))?;

    let host = url
        .host_str()
        .ok_or_else(|| AppError::Validation(format!("URL must have a host: '{raw_url}'")))?;

    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::Forbidden(
            "URLs with credentials embedded are forbidden".into(),
        ));
    }

    let is_allowlisted = allowlist
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(host));

    if require_https && url.scheme() != "https" && !is_allowlisted {
        return Err(AppError::Forbidden(
            "HTTPS is required for external endpoints unless allowlisted".into(),
        ));
    }

    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::Forbidden(format!(
            "Unsupported URL scheme '{}'; only http and https are permitted",
            url.scheme()
        )));
    }

    let port = url.port_or_known_default().ok_or_else(|| {
        AppError::Validation(format!("Could not determine port for URL '{raw_url}'"))
    })?;

    let addresses: Vec<SocketAddr> = if let Ok(ip) = host.parse::<IpAddr>() {
        vec![SocketAddr::new(ip, port)]
    } else {
        tokio::net::lookup_host((host, port))
            .await
            .map_err(|e| {
                AppError::ExternalService(format!("Failed to resolve DNS for {host}: {e}"))
            })?
            .collect()
    };

    if addresses.is_empty() {
        return Err(AppError::Validation(format!(
            "No IP addresses resolved for host '{host}'"
        )));
    }

    if !is_allowlisted {
        for addr in &addresses {
            if is_blocked_ip(addr.ip()) {
                return Err(AppError::Forbidden(format!(
                    "Access to private/local network address '{}' ({}) is forbidden",
                    host,
                    addr.ip()
                )));
            }
        }
    }

    Ok((url, addresses))
}

/// Builds a hardened reqwest::Client that pins connection target IP addresses
/// (preventing DNS rebinding) and strictly disables redirects and proxies.
pub fn build_safe_client(
    host: &str,
    addresses: &[SocketAddr],
) -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .resolve_to_addrs(host, addresses)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build SSRF-safe HTTP client: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_is_blocked_ip_ipv4() {
        // Loopback
        assert!(is_blocked_ip(IpAddr::from_str("127.0.0.1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("127.0.1.1").unwrap()));

        // RFC 1918 Private
        assert!(is_blocked_ip(IpAddr::from_str("10.0.0.1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("10.254.1.1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("172.16.0.1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("172.31.255.255").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("192.168.1.1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("192.168.254.254").unwrap()));

        // Cloud Metadata & Link-local
        assert!(is_blocked_ip(IpAddr::from_str("169.254.169.254").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("169.254.1.1").unwrap()));

        // CGNAT (100.64.0.0/10)
        assert!(is_blocked_ip(IpAddr::from_str("100.64.0.1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("100.127.255.255").unwrap()));

        // Unspecified, Broadcast, Multicast
        assert!(is_blocked_ip(IpAddr::from_str("0.0.0.0").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("255.255.255.255").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("224.0.0.1").unwrap()));

        // Public IPs should NOT be blocked
        assert!(!is_blocked_ip(IpAddr::from_str("8.8.8.8").unwrap()));
        assert!(!is_blocked_ip(IpAddr::from_str("1.1.1.1").unwrap()));
        assert!(!is_blocked_ip(IpAddr::from_str("93.184.216.34").unwrap()));
    }

    #[test]
    fn test_is_blocked_ip_ipv6() {
        // Loopback & Unspecified
        assert!(is_blocked_ip(IpAddr::from_str("::1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("::").unwrap()));

        // Unique local & Link local
        assert!(is_blocked_ip(IpAddr::from_str("fc00::1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("fd00::1").unwrap()));
        assert!(is_blocked_ip(IpAddr::from_str("fe80::1").unwrap()));

        // IPv4-mapped IPv6
        assert!(is_blocked_ip(IpAddr::from_str("::ffff:127.0.0.1").unwrap()));
        assert!(is_blocked_ip(
            IpAddr::from_str("::ffff:169.254.169.254").unwrap()
        ));
        assert!(is_blocked_ip(IpAddr::from_str("::ffff:10.0.0.1").unwrap()));

        // Public IPv6 should NOT be blocked
        assert!(!is_blocked_ip(
            IpAddr::from_str("2606:4700:4700::1111").unwrap()
        ));
        assert!(!is_blocked_ip(
            IpAddr::from_str("2001:4860:4860::8888").unwrap()
        ));
    }

    #[tokio::test]
    async fn test_resolve_and_verify_url_validation() {
        // Loopback URL blocked
        let res = resolve_and_verify_url("http://127.0.0.1:8080/token", false, &[]).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));

        // Metadata IP blocked
        let res =
            resolve_and_verify_url("http://169.254.169.254/latest/meta-data", false, &[]).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));

        // Embedded credentials rejected
        let res = resolve_and_verify_url("https://user:pass@example.com/token", true, &[]).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));

        // Non-http/https scheme rejected
        let res = resolve_and_verify_url("ftp://example.com/resource", false, &[]).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));

        // HTTP rejected when require_https is true
        let res = resolve_and_verify_url("http://example.com/oauth/authorize", true, &[]).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));

        // Allowlisted host permits private IP or HTTP
        let allowlist = vec!["local-idp.test".to_string(), "127.0.0.1".to_string()];
        let res = resolve_and_verify_url("http://127.0.0.1:9090/token", false, &allowlist).await;
        assert!(res.is_ok());
        let (url, addrs) = res.unwrap();
        assert_eq!(url.port(), Some(9090));
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].ip(), IpAddr::from_str("127.0.0.1").unwrap());
    }

    #[test]
    fn test_build_safe_client_construction() {
        let addr = SocketAddr::from_str("93.184.216.34:443").unwrap();
        let client = build_safe_client("example.com", &[addr]);
        assert!(client.is_ok());
    }
}
