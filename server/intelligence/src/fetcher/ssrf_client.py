import ipaddress
import socket
from urllib.parse import urlparse

import httpx
from bs4 import BeautifulSoup

BLOCKED_CIDRS = [
    ipaddress.ip_network("127.0.0.0/8"),
    ipaddress.ip_network("10.0.0.0/8"),
    ipaddress.ip_network("172.16.0.0/12"),
    ipaddress.ip_network("192.168.0.0/16"),
    ipaddress.ip_network("169.254.0.0/16"),
    ipaddress.ip_network("0.0.0.0/8"),
    ipaddress.ip_network("100.64.0.0/10"),
    ipaddress.ip_network("::1/128"),
    ipaddress.ip_network("fc00::/7"),
    ipaddress.ip_network("fe80::/10"),
]


class SSRFSecurityError(ValueError):
    pass


class PlaceholderDomainError(ValueError):
    pass


PLACEHOLDER_DOMAINS = ("example.com", "example.org", "example.net", "localhost")
PLACEHOLDER_TLDS = ("test", "invalid", "example", "localhost")


def reject_placeholder_domain(url: str) -> None:
    hostname = (urlparse(url).hostname or "").rstrip(".").lower()
    if not hostname:
        return
    is_placeholder = any(
        hostname == d or hostname.endswith(f".{d}") for d in PLACEHOLDER_DOMAINS
    ) or hostname.rsplit(".", 1)[-1] in PLACEHOLDER_TLDS
    if is_placeholder:
        raise PlaceholderDomainError(
            f"'{hostname}' is a reserved placeholder domain with no real company content. "
            "Do not fetch it; search for and read the company's real engineering sources instead."
        )


def validate_url_for_ssrf(url: str) -> None:
    parsed = urlparse(url)
    if parsed.scheme not in ("http", "https"):
        raise SSRFSecurityError(f"Unsupported URL scheme: {parsed.scheme}")
    hostname = parsed.hostname
    if not hostname:
        raise SSRFSecurityError("Missing hostname in URL")
    try:
        addr_info = socket.getaddrinfo(
            hostname, parsed.port or (443 if parsed.scheme == "https" else 80)
        )
    except socket.gaierror as e:
        raise SSRFSecurityError(f"Failed to resolve hostname '{hostname}': {e}")
    for entry in addr_info:
        ip_str = entry[4][0]
        ip = ipaddress.ip_address(ip_str)
        for blocked_cidr in BLOCKED_CIDRS:
            if ip in blocked_cidr:
                raise SSRFSecurityError(
                    f"Access to '{hostname}' ({ip}) is blocked: resolves to private/link-local address."
                )


class SafeResearchFetcher:
    def __init__(self, timeout_seconds: float = 10.0, max_bytes: int = 5 * 1024 * 1024):
        self.timeout = timeout_seconds
        self.max_bytes = max_bytes

    async def fetch_clean_text(self, url: str) -> tuple[str, str]:
        validate_url_for_ssrf(url)
        async with httpx.AsyncClient(timeout=self.timeout, follow_redirects=False) as client:
            current_url = url
            for _ in range(3):
                validate_url_for_ssrf(current_url)
                resp = await client.get(
                    current_url,
                    headers={"User-Agent": "MensetsuResearchBot/1.0 (+https://mensetsu.local)"},
                )
                if resp.is_redirect:
                    location = resp.headers.get("Location")
                    if not location:
                        break
                    current_url = str(httpx.URL(current_url).join(location))
                    continue
                resp.raise_for_status()
                break
            import warnings

            from bs4 import XMLParsedAsHTMLWarning

            html = resp.text
            with warnings.catch_warnings():
                warnings.filterwarnings("ignore", category=XMLParsedAsHTMLWarning)
                soup = BeautifulSoup(html, "html.parser")
            for elem in soup(["script", "style", "noscript", "iframe", "svg"]):
                elem.decompose()
            title = soup.title.string.strip() if soup.title and soup.title.string else "Untitled"
            text = soup.get_text(separator="\n", strip=True)
            return (title, text[: self.max_bytes])
