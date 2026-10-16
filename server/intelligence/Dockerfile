FROM ghcr.io/astral-sh/uv:python3.12-bookworm-slim

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

CMD ["python", "-m", "src.main"]
