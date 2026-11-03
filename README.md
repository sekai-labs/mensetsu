# Mensetsu (面接)

[![Container images pipeline](https://github.com/sekai-labs/mensetsu/actions/workflows/images.yml/badge.svg?branch=main)](https://github.com/sekai-labs/mensetsu/actions/workflows/images.yml) [![Native clients pipeline](https://github.com/sekai-labs/mensetsu/actions/workflows/native-clients.yml/badge.svg?branch=main)](https://github.com/sekai-labs/mensetsu/actions/workflows/native-clients.yml)

[![Rust API](https://img.shields.io/badge/API-Rust-555555?logo=rust&logoColor=white)](server/api) [![Python worker](https://img.shields.io/badge/Worker-Python-555555?logo=python&logoColor=white)](server/intelligence) [![Nuxt web](https://img.shields.io/badge/Web-Nuxt-555555?logo=nuxt&logoColor=white)](app/web) [![SurrealDB](https://img.shields.io/badge/Data-SurrealDB-555555)](docker-compose.yml) [![Redis](https://img.shields.io/badge/Jobs-Redis-555555?logo=redis&logoColor=white)](docker-compose.yml)

**Self-hosted, evidence-backed preparation for technical interviews.** Mensetsu brings a résumé, a target role, and company research into one workspace. It connects claims to sources, highlights experience gaps, and turns those gaps into a learning path and interview practice.

The web app and five native clients connect to a Rust API; a separate Python worker runs research and analysis. You deploy the services and control where candidate data is stored.

Build badges report GitHub mirror workflows on `main`; they may not reflect the latest [Codeberg source](https://codeberg.org/sekai-labs/mensetsu).

## What you can do

- Create a workspace from a PDF résumé, job description, company, and target role.
- Review company research with source links and inspect how your experience matches role requirements.
- Explore skills and knowledge relationships, follow a learning path, and practice with targeted questions.
- Track background research runs and return to the workspace as results arrive.

AI-generated material can be wrong. Check important claims against their cited sources before relying on them.

## Quick start: local deployment

**Requirements:** Docker Engine with Compose v2 and credentials for an OpenAI-compatible model endpoint. The supplied Compose configuration is for local development, **not** an internet-facing production deployment.

1. Create a local configuration file:

   ```bash
   cp .env.example .env
   ```

2. Edit `.env`. Set unique values for `JWT_SECRET`, `ENCRYPTION_KEY`, `INTERNAL_SERVICE_TOKEN`, and `SEARXNG_SECRET`. Set `LLM_OPENAI_BASE_URL`, `LLM_OPENAI_API_KEY`, and `LLM_DEFAULT_MODEL` for the model you intend to use. The worker refuses to start without the model base URL and key. The template's MinIO credentials are development defaults; replace them for any shared deployment.

3. Build and start the services:

   ```bash
   docker compose up -d --build
   ```

4. Open **http://localhost:3001** and complete the first-run administrator setup. The API health check is **http://localhost:3000/health/live**. If a service does not start, inspect it with `docker compose ps` and `docker compose logs <service>`.

Stop the stack with `docker compose down`. Do **not** add `-v` unless you intend to delete the database, Redis, and object-storage volumes.

### Hosting beyond localhost

The sample Compose file publishes the API, SurrealDB, and Redis ports and configures development database credentials. Do not expose it unchanged to the internet. For a shared or public deployment, configure TLS and allowed origins, restrict internal service access, replace all sample credentials, protect the object store, and arrange persistent storage and backups. Uploaded résumés contain personal data; secure and retain them accordingly.

PDF uploads use short-lived signed URLs to private S3-compatible storage. Set `S3_ENDPOINT_PUBLIC` to an HTTPS endpoint reachable by the **browser or native device** at the same hostname used in the signed URL; the default `localhost:9000` only works on the machine running the local stack. See the [system architecture](docs/architecture/system-overview.md) for service boundaries.

**Existing deployments:** If your database contains legacy inline/base64 PDF résumés, run [`infra/verify_and_migrate.sh`](infra/verify_and_migrate.sh) with the matching database and object-store configuration **before starting the updated API**. It verifies each migrated object before dropping the old field and stops on a missing or corrupt record. Back up your data first.

## Native clients

Android, iOS, Linux, macOS, and Windows clients are in [`app/mobile`](app/mobile) and [`app/desktop`](app/desktop). On first launch, enter the **Rust API origin**, for example `https://api.example.org`—not the web app URL or an `/api/v1` path. Remote servers require HTTPS; loopback HTTP is for local development. `localhost` on a phone or another computer refers to that device, not your server. You can change servers from settings or the sign-in screen; switching signs out the current session.

| Platform | Source | Build requirements |
| --- | --- | --- |
| Android | [`app/mobile/android`](app/mobile/android) | JDK 17, Android SDK 34, Gradle |
| iOS | [`app/mobile/ios`](app/mobile/ios) | Xcode on macOS |
| Linux | [`app/desktop/linux`](app/desktop/linux) | Rust, GTK4, libsecret |
| macOS | [`app/desktop/macos`](app/desktop/macos) | Xcode on macOS |
| Windows | [`app/desktop/windows`](app/desktop/windows) | Visual Studio 2022, Windows SDK, .NET 9 SDK |

Apple distribution builds require your own signing and provisioning. Native builds use generated API clients from [`contracts/native`](contracts/native).

## Architecture

| Component | Responsibility |
| --- | --- |
| [`app/web`](app/web) | Nuxt web UI; proxies API requests and manages browser sessions |
| [`server/api`](server/api) | Rust API; identity, authorization, workspaces, storage access, and OpenAPI |
| [`server/intelligence`](server/intelligence) | Python intelligence service and worker; research and analysis |
| [`contracts`](contracts) | REST/OpenAPI and internal gRPC contracts; generated client code |
| SurrealDB, Redis, SearXNG, S3-compatible storage | Workspace data, job coordination, web search, and private PDF objects |

The [architecture overview](docs/architecture/system-overview.md) covers service communication and workflow execution.

## Development and verification

Local development requires Rust, Bun, Python 3.12+, `uv`, and Docker Compose for supporting services. Copy and configure `.env` as above. `make infra-up` starts SurrealDB and Redis; research and uploads additionally need SearXNG and MinIO (`docker compose up -d searxng minio minio-init`). Start the applications in separate terminals with `make dev-api`, `make dev-ai`, `make dev-worker`, and `make dev-web`.

Run `make test` for the API, intelligence, and web suites. Run `make contracts` after changing the Rust REST contract or gRPC definitions; `make contracts-check` verifies generated output. `make check` also checks schema parity against a running SurrealDB instance and performs source checks. Contract generation requires Docker and OpenAPI Generator.

**Warning:** `make e2e` resets its configured SurrealDB database and flushes Redis. Use a disposable environment only. To inspect the reset before running the browser suite:

```bash
RESET_DRY_RUN=1 MENSETSU_ALLOW_RESET=1 make e2e-reset
MENSETSU_ALLOW_RESET=1 make e2e
```

## Contributing

Issues and pull requests are welcome. Include a reproducible description, the affected client or service, and the checks you ran. For API changes, regenerate and include the affected contracts and clients. Do not submit real résumés, credentials, or other personal data in issues or test fixtures; use synthetic data.

## License

Mensetsu is licensed under [Apache-2.0](LICENSE).
