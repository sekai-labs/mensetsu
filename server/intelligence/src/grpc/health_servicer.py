import logging

import grpc
from src.db.client import SurrealDirectClient
from src.grpc.v1 import health_pb2, health_pb2_grpc
from src.orchestration.shared import job_queue

logger = logging.getLogger("mensetsu.grpc.health")


class HealthServicer(health_pb2_grpc.HealthServicer):
    async def Check(
        self,
        request: health_pb2.HealthCheckRequest,
        context: grpc.aio.ServicerContext,
    ) -> health_pb2.HealthCheckResponse:
        surreal_ok = False
        redis_ok = False

        try:
            surreal_client = SurrealDirectClient()
            await surreal_client.execute_query("INFO FOR DB;")
            surreal_ok = True
        except Exception as e:
            logger.warning(f"Health check: SurrealDB error: {e}")

        try:
            client = await job_queue.get_client()
            await client.ping()
            redis_ok = True
        except Exception as e:
            logger.warning(f"Health check: Redis error: {e}")

        if surreal_ok and redis_ok:
            return health_pb2.HealthCheckResponse(
                status=health_pb2.HealthCheckResponse.ServingStatus.SERVING
            )
        return health_pb2.HealthCheckResponse(
            status=health_pb2.HealthCheckResponse.ServingStatus.NOT_SERVING
        )

    async def Watch(
        self,
        request: health_pb2.HealthCheckRequest,
        context: grpc.aio.ServicerContext,
    ):
        resp = await self.Check(request, context)
        yield resp
