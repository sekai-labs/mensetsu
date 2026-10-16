#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
PROTO_DIR="${ROOT_DIR}/contracts/grpc/proto/v1"
PY_OUT_DIR="${ROOT_DIR}/server/intelligence/src/grpc/v1"

mkdir -p "${PY_OUT_DIR}"
touch "${PY_OUT_DIR}/__init__.py"
touch "${ROOT_DIR}/server/intelligence/src/grpc/__init__.py"

echo "Compiling Protobuf definitions to Python stubs in ${PY_OUT_DIR}..."

cd "${ROOT_DIR}/server/intelligence"
uv run python -m grpc_tools.protoc \
  -I"${PROTO_DIR}" \
  --python_out="${PY_OUT_DIR}" \
  --grpc_python_out="${PY_OUT_DIR}" \
  --pyi_out="${PY_OUT_DIR}" \
  "${PROTO_DIR}"/health.proto \
  "${PROTO_DIR}"/control.proto \
  "${PROTO_DIR}"/streaming.proto

# Fix relative imports in generated grpc files if needed
sed -i -E 's/^import ([a-zA-Z0-9_]+_pb2) as/from . import \1 as/' "${PY_OUT_DIR}"/*_pb2_grpc.py 2>/dev/null || true

echo "Protobuf Python stubs generated successfully."
