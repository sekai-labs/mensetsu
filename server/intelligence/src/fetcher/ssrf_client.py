import ipaddress
import socket
import warnings
import zlib
from urllib.parse import urlparse

import httpx
from bs4 import BeautifulSoup, XMLParsedAsHTMLWarning
from httpcore._backends.auto import AutoBackend
from httpcore._backends.base import AsyncNetworkStream

BLOCKED_CIDRS = [
    ipaddress.ip_network("127.0.0.0/8"), ipaddress.ip_network("10.0.0.0/8"),
    ipaddress.ip_network("172.16.0.0/12"), ipaddress.ip_network("192.168.0.0/16"),
    ipaddress.ip_network("169.254.0.0/16"), ipaddress.ip_network("0.0.0.0/8"),
    ipaddress.ip_network("100.64.0.0/10"), ipaddress.ip_network("::1/128"),
    ipaddress.ip_network("fc00::/7"), ipaddress.ip_network("fe80::/10"),
]


class SSRFSecurityError(ValueError):
    pass


class ResponseTooLargeError(ValueError):
    pass


class PlaceholderDomainError(ValueError):
    pass


PLACEHOLDER_DOMAINS = ("example.com", "example.org", "example.net", "localhost")
PLACEHOLDER_TLDS = ("test", "invalid", "example", "localhost")


def reject_placeholder_domain(url: str) -> None:
    hostname = (urlparse(url).hostname or "").rstrip(".").lower()
    if not hostname:
        return
    is_placeholder = (
        any(hostname == d or hostname.endswith(f".{d}") for d in PLACEHOLDER_DOMAINS)
        or hostname.rsplit(".", 1)[-1] in PLACEHOLDER_TLDS
    )
    if is_placeholder:
        raise PlaceholderDomainError(
            f"'{hostname}' is a reserved placeholder domain with no real company content. "
            "Do not fetch it; search for and read the company's real engineering sources instead."
        )


def resolve_public_addresses(url: str) -> tuple[str, ...]:
    parsed = urlparse(url)
    if parsed.scheme not in ("http", "https"):
        raise SSRFSecurityError(f"Unsupported URL scheme: {parsed.scheme}")
    hostname = parsed.hostname
    if not hostname:
        raise SSRFSecurityError("Missing hostname in URL")
    try:
        answers = socket.getaddrinfo(
            hostname, parsed.port or (443 if parsed.scheme == "https" else 80), type=socket.SOCK_STREAM
        )
    except socket.gaierror as error:
        raise SSRFSecurityError(f"Failed to resolve hostname '{hostname}': {error}") from error
    addresses = tuple(dict.fromkeys(entry[4][0].split("%", 1)[0] for entry in answers))
    if not addresses:
        raise SSRFSecurityError(f"Hostname '{hostname}' returned no addresses")
    for address in addresses:
        ip = ipaddress.ip_address(address)
        if isinstance(ip, ipaddress.IPv6Address) and ip.ipv4_mapped is not None:
            ip = ip.ipv4_mapped
        if (
            not ip.is_global
            or ip.is_unspecified
            or ip.is_multicast
            or any(ip in network for network in BLOCKED_CIDRS)
        ):
            raise SSRFSecurityError(
                f"Access to '{hostname}' ({ip}) is blocked: address is non-global, private, or reserved."
            )
    return addresses


def validate_url_for_ssrf(url: str) -> None:
    resolve_public_addresses(url)


class _PinnedNetworkBackend:
    def __init__(self, hostname: str, addresses: tuple[str, ...]):
        self.hostname = hostname.lower().rstrip(".")
        self.addresses = addresses
        self.backend = AutoBackend()

    async def connect_tcp(self, host: str, port: int, **kwargs) -> AsyncNetworkStream:
        if host.lower().rstrip(".") != self.hostname:
            raise SSRFSecurityError(f"Unexpected connection host: {host}")
        last_error = None
        for address in self.addresses:
            try:
                return await self.backend.connect_tcp(address, port, **kwargs)
            except Exception as error:
                last_error = error
        raise last_error

    async def connect_unix_socket(self, path: str, **kwargs):
        raise SSRFSecurityError("Unix socket connections are not permitted")

    async def sleep(self, seconds: float) -> None:
        await self.backend.sleep(seconds)

class _LimitedContentDecoder:
    """Incremental gzip/deflate decoder with bounded output."""

    def __init__(self, encoding: str):
        self.encoding = encoding
        self.raw_deflate = False
        self.decoder = self._new_decoder()

    def _new_decoder(self):
        if self.encoding == "gzip":
            return zlib.decompressobj(16 + zlib.MAX_WBITS)
        return zlib.decompressobj()

    def decode(self, data: bytes, output_limit: int) -> bytes:
        try:
            return self.decoder.decompress(data, output_limit)
        except zlib.error:
            if self.encoding != "deflate" or self.raw_deflate:
                raise
            self.raw_deflate = True
            self.decoder = zlib.decompressobj(-zlib.MAX_WBITS)
            return self.decoder.decompress(data, output_limit)

    def flush(self, output_limit: int) -> bytes:
        return self.decoder.flush(output_limit)



def pinned_async_client(url: str, *, timeout: float) -> httpx.AsyncClient:
    parsed = urlparse(url)
    addresses = resolve_public_addresses(url)
    # HTTPX exposes no resolver hook; private backend access is version-pinned and tested.
    transport = httpx.AsyncHTTPTransport(trust_env=False)
    transport._pool._network_backend = _PinnedNetworkBackend(parsed.hostname, addresses)
    return httpx.AsyncClient(
        timeout=timeout,
        follow_redirects=False,
        trust_env=False,
        transport=transport,
    )


async def fetch_limited_response(
    url: str,
    *,
    timeout: float,
    headers: dict[str, str],
    max_bytes: int,
    max_encoded_bytes: int | None = None,
) -> httpx.Response:
    encoded_limit = max_bytes if max_encoded_bytes is None else max_encoded_bytes
    request_headers = httpx.Headers(headers)
    if "accept-encoding" not in request_headers:
        request_headers["Accept-Encoding"] = "gzip, deflate"
    async with pinned_async_client(url, timeout=timeout) as client:
        async with client.stream("GET", url, headers=request_headers) as response:
            encoding = response.headers.get("content-encoding", "identity").strip().lower()
            if encoding not in {"identity", "gzip", "deflate"}:
                raise ValueError(f"Unsupported Content-Encoding: {encoding}")
            decoder = None if encoding == "identity" else _LimitedContentDecoder(encoding)
            chunks: list[bytes] = []
            encoded_size = 0
            decoded_size = 0
            async for raw_chunk in response.aiter_raw(
                chunk_size=min(64 * 1024, encoded_limit + 1)
            ):
                encoded_size += len(raw_chunk)
                if encoded_size > encoded_limit:
                    raise ResponseTooLargeError(
                        f"Encoded response exceeds {encoded_limit} byte limit"
                    )
                decoded_chunk = (
                    raw_chunk
                    if decoder is None
                    else decoder.decode(raw_chunk, max_bytes - decoded_size + 1)
                )
                decoded_size += len(decoded_chunk)
                if decoded_size > max_bytes:
                    raise ResponseTooLargeError(
                        f"Decoded response exceeds {max_bytes} byte limit"
                    )
                chunks.append(decoded_chunk)
            if decoder is not None:
                final_chunk = decoder.flush(max_bytes - decoded_size + 1)
                decoded_size += len(final_chunk)
                if decoded_size > max_bytes:
                    raise ResponseTooLargeError(
                        f"Decoded response exceeds {max_bytes} byte limit"
                    )
                chunks.append(final_chunk)
            response_headers = httpx.Headers(response.headers)
            response_headers.pop("content-encoding", None)
            response_headers.pop("content-length", None)
            return httpx.Response(
                response.status_code,
                headers=response_headers,
                content=b"".join(chunks),
                request=response.request,
            )


class SafeResearchFetcher:
    def __init__(self, timeout_seconds: float = 10.0, max_bytes: int = 5 * 1024 * 1024):
        self.timeout = timeout_seconds
        self.max_bytes = max_bytes

    async def fetch_clean_text(self, url: str) -> tuple[str, str]:
        current_url = url
        for _ in range(3):
            response = await fetch_limited_response(
                current_url, timeout=self.timeout,
                headers={"User-Agent": "MensetsuResearchBot/1.0 (+https://mensetsu.local)"},
                max_bytes=self.max_bytes,
            )
            if response.is_redirect:
                location = response.headers.get("Location")
                if not location:
                    break
                current_url = str(httpx.URL(current_url).join(location))
                continue
            response.raise_for_status()
            break
        
        with warnings.catch_warnings():
            warnings.filterwarnings("ignore", category=XMLParsedAsHTMLWarning)
            soup = BeautifulSoup(response.text, "html.parser")
        for elem in soup(["script", "style", "noscript", "iframe", "svg"]):
            elem.decompose()
        title = soup.title.string.strip() if soup.title and soup.title.string else "Untitled"
        return title, soup.get_text(separator="\n", strip=True)
