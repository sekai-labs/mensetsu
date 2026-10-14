import pytest
from httpx import ASGITransport, AsyncClient

from src.service import app


@pytest.mark.asyncio
async def test_internal_auth_matrix():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        from src.orchestration.router import router as workflow_router

        internal_routes = [r.path for r in workflow_router.routes]
        assert len(internal_routes) > 0, "No internal routes found"

        sample_route = "/internal/v1/workflows"

        res = await client.get(sample_route)
        assert res.status_code == 401

        res = await client.get(sample_route, headers={"Authorization": "Bearer wrong_token_12345"})
        assert res.status_code == 401

        res = await client.get(sample_route, headers={"Authorization": "Basic dXNlcjpwYXNz"})
        assert res.status_code == 401

        res = await client.get(sample_route, headers={"Authorization": "Bearer "})
        assert res.status_code == 401


@pytest.mark.asyncio
async def test_health_probes():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        res = await client.get("/health/live")
        assert res.status_code == 200
        assert res.json() == {"status": "alive"}
