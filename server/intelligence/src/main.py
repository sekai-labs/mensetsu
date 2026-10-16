import asyncio
import logging
import os

import grpc
from src.grpc.auth_interceptor import GrpcAuthInterceptor
from src.grpc.control_servicer import DataControlServicer
from src.grpc.health_servicer import HealthServicer
from src.grpc.streaming_servicer import WorkflowEventStreamServicer
from src.grpc.v1 import control_pb2_grpc, health_pb2_grpc, streaming_pb2_grpc

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(name)s: %(message)s")
logger = logging.getLogger("mensetsu.intelligence.server")


def create_grpc_server() -> grpc.aio.Server:
    # Enforce fail-closed startup invariant
    from src.orchestration.auth import get_internal_token

    get_internal_token()

    server = grpc.aio.server(
        interceptors=[GrpcAuthInterceptor()],
        options=[
            ("grpc.max_send_message_length", 64 * 1024 * 1024),
            ("grpc.max_receive_message_length", 64 * 1024 * 1024),
            ("grpc.keepalive_time_ms", 30000),
            ("grpc.keepalive_timeout_ms", 10000),
            ("grpc.keepalive_permit_without_calls", True),
            ("grpc.http2.max_pings_without_data", 0),
        ],
    )
    control_pb2_grpc.add_DataControlServiceServicer_to_server(DataControlServicer(), server)
    streaming_pb2_grpc.add_WorkflowEventStreamServiceServicer_to_server(
        WorkflowEventStreamServicer(), server
    )
    health_pb2_grpc.add_HealthServicer_to_server(HealthServicer(), server)
    return server


async def serve_grpc():
    host = os.environ.get("INTELLIGENCE_HOST", "0.0.0.0")
    port = int(os.environ.get("INTELLIGENCE_PORT", "50051"))
    bind_addr = f"{host}:{port}"

    server = create_grpc_server()
    server.add_insecure_port(bind_addr)

    logger.info(f"Starting Mensetsu Intelligence gRPC service on {bind_addr}...")
    await server.start()
    logger.info("Intelligence gRPC service started and accepting connections.")

    try:
        await server.wait_for_termination()
    except asyncio.CancelledError:
        logger.info("Termination signal received. Gracefully stopping gRPC server...")
        await server.stop(grace=5.0)


def main():
    asyncio.run(serve_grpc())


if __name__ == "__main__":
    main()
