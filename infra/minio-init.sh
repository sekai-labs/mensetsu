#!/bin/sh
set -e
export MC_CONFIG_DIR=/tmp/minio-client

echo "Configuring MinIO client alias..."
attempt=0
until mc alias set local http://minio:9000 "${AWS_ACCESS_KEY_ID:-minioadmin}" "${AWS_SECRET_ACCESS_KEY:-minioadmin}"; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 30 ]; then
    echo "MinIO did not accept credentials after 30 attempts" >&2
    exit 1
  fi
  sleep 2
done

BUCKET_NAME="${S3_BUCKET:-mensetsu-resumes}"
echo "Ensuring private bucket ${BUCKET_NAME} exists..."
mc mb --ignore-existing "local/${BUCKET_NAME}"

echo "Enforcing private access (no anonymous read/download)..."
mc anonymous set none "local/${BUCKET_NAME}"

echo "Private bucket initialized; MinIO CORS is configured at the server origin."
