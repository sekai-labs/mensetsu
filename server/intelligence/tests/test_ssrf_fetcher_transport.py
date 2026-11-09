import asyncio
import gzip
import socket
import ssl
from pathlib import Path

import httpx
import pytest

import src.fetcher.ssrf_client as fetcher_module
from src.fetcher.ssrf_client import (
    ResponseTooLargeError,
    SafeResearchFetcher,
    SSRFSecurityError,
    fetch_limited_response,
)

PUBLIC_IP = "8.8.8.8"
HOST = "fetch-test.invalid"


class CountingStream:
    def __init__(self, stream, read_counts):
        self.stream = stream
        self.read_counts = read_counts

    async def read(self, max_bytes, timeout=None):
        data = await self.stream.read(max_bytes, timeout=timeout)
        self.read_counts[0] += len(data)
        return data

    async def write(self, buffer, timeout=None):
        await self.stream.write(buffer, timeout=timeout)

    async def aclose(self):
        await self.stream.aclose()

    async def start_tls(self, ssl_context, server_hostname=None, timeout=None):
        stream = await self.stream.start_tls(
            ssl_context, server_hostname=server_hostname, timeout=timeout
        )
        return CountingStream(stream, self.read_counts)

    def get_extra_info(self, info):
        return self.stream.get_extra_info(info)


class LocalBackend:
    """Map the pinned target to a local listener and count socket bytes consumed."""

    def __init__(self, real_backend, targets, read_counts):
        self.real_backend = real_backend
        self.targets = targets
        self.read_counts = read_counts

    async def connect_tcp(self, host, port, **kwargs):
        self.targets.append(host)
        stream = await self.real_backend.connect_tcp("127.0.0.1", port, **kwargs)
        return CountingStream(stream, self.read_counts)

    async def connect_unix_socket(self, path, **kwargs):
        raise AssertionError("Unix sockets are not part of this fetch path")

    async def sleep(self, seconds):
        await self.real_backend.sleep(seconds)


async def run_http_server(handler):
    server = await asyncio.start_server(handler, "127.0.0.1", 0)
    return server, server.sockets[0].getsockname()[1]


def use_local_backend(monkeypatch):
    actual_backend_type = fetcher_module.AutoBackend
    targets = []
    read_counts = [0]

    def backend_factory():
        return LocalBackend(actual_backend_type(), targets, read_counts)

    monkeypatch.setattr(fetcher_module, "AutoBackend", backend_factory)
    return targets, read_counts


def resolver_with_answers(monkeypatch, answers):
    original_getaddrinfo = socket.getaddrinfo
    lookups = []

    def getaddrinfo(host, port, *args, **kwargs):
        if host in answers:
            lookups.append(host)
            result = answers[host]
            if isinstance(result, list):
                result = result[min(lookups.count(host) - 1, len(result) - 1)]
            return [(socket.AF_INET, socket.SOCK_STREAM, socket.IPPROTO_TCP, "", (result, port))]
        return original_getaddrinfo(host, port, *args, **kwargs)

    monkeypatch.setattr(fetcher_module.socket, "getaddrinfo", getaddrinfo)
    return lookups


@pytest.mark.asyncio
async def test_connection_uses_validated_ip_despite_later_private_dns_answer(monkeypatch):
    requested_targets, _ = use_local_backend(monkeypatch)
    lookup_count = resolver_with_answers(monkeypatch, {HOST: [PUBLIC_IP, "127.0.0.1"]})

    async def respond(reader, writer):
        await reader.readuntil(b"\r\n\r\n")
        writer.write(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
        await writer.drain()
        writer.close()

    server, port = await run_http_server(respond)
    try:
        response = await fetch_limited_response(
            f"http://{HOST}:{port}/", timeout=2, headers={}, max_bytes=32
        )
        assert response.content == b"ok"
        assert lookup_count == [HOST]
        assert requested_targets == [PUBLIC_IP]
    finally:
        server.close()
        await server.wait_closed()


@pytest.mark.asyncio
async def test_redirect_to_private_address_is_rejected_before_connect(monkeypatch):
    requested_targets, _ = use_local_backend(monkeypatch)
    resolver_with_answers(monkeypatch, {HOST: PUBLIC_IP, "redirect.invalid": "169.254.169.254"})

    async def respond(reader, writer):
        await reader.readuntil(b"\r\n\r\n")
        writer.write(
            b"HTTP/1.1 302 Found\r\nLocation: http://redirect.invalid/latest/meta-data/\r\n"
            b"Content-Length: 0\r\nConnection: close\r\n\r\n"
        )
        await writer.drain()
        writer.close()

    server, port = await run_http_server(respond)
    try:
        with pytest.raises(SSRFSecurityError, match="non-global"):
            await fetcher_module.SafeResearchFetcher().fetch_clean_text(f"http://{HOST}:{port}/")
        assert requested_targets == [PUBLIC_IP]
    finally:
        server.close()
        await server.wait_closed()


@pytest.mark.asyncio
async def test_https_sends_original_sni_and_rejects_untrusted_certificate(monkeypatch):
    requested_targets, _ = use_local_backend(monkeypatch)
    resolver_with_answers(monkeypatch, {HOST: PUBLIC_IP})
    sni_names = []
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    fixture = Path(__file__).parent / "fixtures"
    context.load_cert_chain(fixture / "fetch-test.crt", fixture / "fetch-test.key")
    context.set_servername_callback(lambda _ssl, name, _ctx: sni_names.append(name))

    async def respond(reader, writer):
        try:
            await reader.read(4096)
        except Exception:
            pass
        writer.close()

    server = await asyncio.start_server(respond, "127.0.0.1", 0, ssl=context)
    port = server.sockets[0].getsockname()[1]
    try:
        with pytest.raises(httpx.ConnectError, match="certificate|SSL|CERTIFICATE"):
            await fetch_limited_response(
                f"https://{HOST}:{port}/", timeout=2, headers={}, max_bytes=32
            )
        assert sni_names == [HOST]
        assert requested_targets == [PUBLIC_IP]
    finally:
        server.close()
        await server.wait_closed()


@pytest.mark.asyncio
async def test_chunked_oversized_body_stops_when_client_consumption_is_bounded(monkeypatch):
    requested_targets, read_counts = use_local_backend(monkeypatch)
    resolver_with_answers(monkeypatch, {HOST: PUBLIC_IP})
    oversized_chunk = b"z" * 1_048_576

    async def respond(reader, writer):
        await reader.readuntil(b"\r\n\r\n")
        writer.write(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
            + f"{len(oversized_chunk):x}\r\n".encode()
            + oversized_chunk
            + b"\r\n0\r\n\r\n"
        )
        await writer.drain()
        writer.close()

    server, port = await run_http_server(respond)
    try:
        with pytest.raises(ResponseTooLargeError, match="5 byte limit"):
            await fetch_limited_response(
                f"http://{HOST}:{port}/", timeout=2, headers={}, max_bytes=5
            )
        assert requested_targets == [PUBLIC_IP]
        assert read_counts[0] < len(oversized_chunk)
    finally:
        server.close()
        await server.wait_closed()


@pytest.mark.asyncio
async def test_gzip_html_is_transparently_decoded(monkeypatch):
    use_local_backend(monkeypatch)
    resolver_with_answers(monkeypatch, {HOST: PUBLIC_IP})
    compressed = gzip.compress(
        b"<html><head><title>Compressed</title></head><body>Readable page</body></html>"
    )

    async def respond(reader, writer):
        await reader.readuntil(b"\r\n\r\n")
        writer.write(
            b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\n"
            + f"Content-Length: {len(compressed)}\r\nConnection: close\r\n\r\n".encode()
            + compressed
        )
        await writer.drain()
        writer.close()

    server, port = await run_http_server(respond)
    try:
        title, text = await SafeResearchFetcher().fetch_clean_text(
            f"http://{HOST}:{port}/"
        )
        assert title == "Compressed"
        assert "Readable page" in text
    finally:
        server.close()
        await server.wait_closed()


@pytest.mark.asyncio
async def test_gzip_decoded_size_cap_rejects_compression_bomb(monkeypatch):
    use_local_backend(monkeypatch)
    resolver_with_answers(monkeypatch, {HOST: PUBLIC_IP})
    compressed = gzip.compress(b"x" * 100_000)
    assert len(compressed) < 512

    async def respond(reader, writer):
        await reader.readuntil(b"\r\n\r\n")
        writer.write(
            b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\n"
            + f"Content-Length: {len(compressed)}\r\nConnection: close\r\n\r\n".encode()
            + compressed
        )
        await writer.drain()
        writer.close()

    server, port = await run_http_server(respond)
    try:
        with pytest.raises(ResponseTooLargeError, match="Decoded response exceeds 128"):
            await fetch_limited_response(
                f"http://{HOST}:{port}/", timeout=2, headers={}, max_bytes=128,
                max_encoded_bytes=512,
            )
    finally:
        server.close()
        await server.wait_closed()
