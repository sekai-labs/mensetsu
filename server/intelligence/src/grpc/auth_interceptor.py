import hmac
import os
from typing import Callable

import grpc


def get_internal_token() -> str:
    token = os.environ.get("INTERNAL_SERVICE_TOKEN")
    if not token or not token.strip():
        raise RuntimeError(
            "INTERNAL_SERVICE_TOKEN is unset or empty. Service refusing to start (fail-closed)."
        )
    return token.strip()


def validate_token(token_str: str | None) -> bool:
    if not token_str:
        return False
    try:
        expected_token = get_internal_token()
    except RuntimeError:
        return False
    parts = token_str.split(" ", 1)
    if len(parts) != 2 or parts[0].lower() != "bearer":
        return False
    provided_token = parts[1].strip()
    return bool(provided_token and hmac.compare_digest(provided_token, expected_token))


class GrpcAuthInterceptor(grpc.aio.ServerInterceptor):
    def __init__(self, exempt_methods: set[str] | None = None):
        self.exempt_methods = exempt_methods or {
            "/grpc.health.v1.Health/Check",
            "/grpc.health.v1.Health/Watch",
        }

    async def intercept_service(
        self,
        continuation: Callable[[grpc.HandlerCallDetails], grpc.RpcMethodHandler],
        handler_call_details: grpc.HandlerCallDetails,
    ) -> grpc.RpcMethodHandler:
        method = handler_call_details.method
        if method in self.exempt_methods:
            return await continuation(handler_call_details)

        handler = await continuation(handler_call_details)
        if handler is None:
            return None

        # Inspect metadata
        metadata = dict(handler_call_details.invocation_metadata or ())
        auth_header = metadata.get("authorization")

        if not validate_token(auth_header):
            if handler.request_streaming and handler.response_streaming:

                async def abort_stream_stream(request_iterator, context):
                    await context.abort(
                        grpc.StatusCode.UNAUTHENTICATED,
                        "Missing or invalid internal authorization token",
                    )

                return grpc.stream_stream_rpc_method_handler(
                    abort_stream_stream,
                    request_deserializer=handler.request_deserializer,
                    response_serializer=handler.response_serializer,
                )
            elif handler.request_streaming and not handler.response_streaming:

                async def abort_stream_unary(request_iterator, context):
                    await context.abort(
                        grpc.StatusCode.UNAUTHENTICATED,
                        "Missing or invalid internal authorization token",
                    )

                return grpc.stream_unary_rpc_method_handler(
                    abort_stream_unary,
                    request_deserializer=handler.request_deserializer,
                    response_serializer=handler.response_serializer,
                )
            elif not handler.request_streaming and handler.response_streaming:

                async def abort_unary_stream(request, context):
                    await context.abort(
                        grpc.StatusCode.UNAUTHENTICATED,
                        "Missing or invalid internal authorization token",
                    )

                return grpc.unary_stream_rpc_method_handler(
                    abort_unary_stream,
                    request_deserializer=handler.request_deserializer,
                    response_serializer=handler.response_serializer,
                )
            else:

                async def abort_unary_unary(request, context):
                    await context.abort(
                        grpc.StatusCode.UNAUTHENTICATED,
                        "Missing or invalid internal authorization token",
                    )

                return grpc.unary_unary_rpc_method_handler(
                    abort_unary_unary,
                    request_deserializer=handler.request_deserializer,
                    response_serializer=handler.response_serializer,
                )

        return handler
