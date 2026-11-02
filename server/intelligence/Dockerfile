FROM python:3.12-slim-trixie AS runtime

RUN pip install --no-cache-dir uv==0.12.3

ENV UV_COMPILE_BYTECODE=1
ENV UV_LINK_MODE=copy

WORKDIR /app

RUN useradd -m -u 1000 appuser

COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-dev --no-install-project

COPY src ./src
COPY agents ./agents
RUN uv sync --frozen --no-dev
RUN chown -R appuser:appuser /app
USER appuser
ENV PYTHONPATH="/app"
ENV PATH="/app/.venv/bin:$PATH"
ENV INTELLIGENCE_PORT=50051
ENV INTELLIGENCE_HOST=0.0.0.0

EXPOSE 50051
FROM runtime AS intelligence
CMD ["python", "-m", "src.main"]

FROM runtime AS worker
CMD ["python", "-m", "src.worker"]
