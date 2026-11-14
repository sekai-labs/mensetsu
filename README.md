# Mensetsu (面接)

![Container images pipeline](https://github.com/sekai-labs/mensetsu/actions/workflows/images.yml/badge.svg?branch=main) ![Native clients pipeline](https://github.com/sekai-labs/mensetsu/actions/workflows/native-clients.yml/badge.svg?branch=main)

![Rust API](https://img.shields.io/badge/API-Rust-555555?logo=rust&logoColor=white) ![Python worker](https://img.shields.io/badge/Worker-Python-555555?logo=python&logoColor=white) ![Nuxt web](https://img.shields.io/badge/Web-Nuxt-555555?logo=nuxt&logoColor=white) ![SurrealDB](https://img.shields.io/badge/Data-SurrealDB-555555) ![Redis](https://img.shields.io/badge/Jobs-Redis-555555?logo=redis&logoColor=white)

**Self-hosted, evidence-backed preparation for technical interviews.** Mensetsu brings a résumé, a target role, and company research into one workspace. It connects claims to sources, highlights experience gaps, and turns those gaps into a learning path and interview practice.

The web app and five native clients connect to a Rust API; a separate Python worker runs research and analysis. You deploy the services and control where candidate data is stored.

## What you can do

- Create a workspace from a PDF résumé, job description, company, and target role.
- Keep reusable PDF résumés and candidate contact details, role targets, and location/keyword preferences in the candidate profile.
- Prepare and track job application records from supported job pages; review captured details and application fields before the extension fills anything.
- Review company research with source links and inspect how your experience matches role requirements.
- Explore skills and knowledge relationships, follow a learning path, and practice with targeted questions.
- Track background research runs and return to the workspace as results arrive.
- Monitor per-request model token usage; users see only their own requests, and administrators can inspect usage across the platform.

AI-generated material can be wrong. Check important claims against their cited sources before relying on them.

## Quick start: local deployment

**Requirements:** Docker Engine with Compose v2 and credentials for an OpenAI-compatible model endpoint. The supplied Compose configuration is for local development, **not** an internet-facing production deployment.

1. Create a local configuration file:
   ```bash
    cp .env.example .env
   ```

   
2. Generate independent secrets and place them in `.env`:
   ```bash
    printf 'JWT_SECRET=%s\n' "$(openssl rand -hex 32)"
    printf 'ENCRYPTION_KEY=%s\n' "$(openssl rand -hex 32)"
   ```

    Copy each command's output into its corresponding `.env` entry; do not reuse either secret. Set a unique `INTERNAL_SERVICE_TOKEN` and `SEARXNG_SECRET` too. Set `LLM_OPENAI_BASE_URL`, `LLM_OPENAI_API_KEY`, and `LLM_DEFAULT_MODEL` for the model you intend to use. The worker refuses to start without the model base URL and key. The template's MinIO credentials are development defaults; replace them for any shared deployment.
3. Build and start the services:
   ```bash
    docker compose up -d --build
   ```

   
4. Open the public landing and documentation at [**http://localhost:3002**](http://localhost:3002) and the interactive candidate workspace at [**http://localhost:3001**](http://localhost:3001). Create the first administrator through **POST `/api/v1/setup/initialize`**. The API health check is [**http://localhost:3000/health/live**](http://localhost:3000/health/live). If a service does not start, inspect it with `docker compose ps` and `docker compose logs <service>`.

Stop the stack with `docker compose down`. Do **not** add `-v` unless you intend to delete the database, Redis, and object-storage volumes.

### Hosting beyond localhost

The sample Compose file publishes the API, SurrealDB, and Redis ports and configures development database credentials. Do not expose it unchanged to the internet. For a shared or public deployment, configure TLS and allowed origins, restrict internal service access, replace all sample credentials, protect the object store, and arrange persistent storage and backups. Uploaded résumés contain personal data; secure and retain them accordingly.

PDF uploads use short-lived signed URLs to private S3-compatible storage. Set `S3_ENDPOINT_PUBLIC` to an HTTPS endpoint reachable by the **browser or native device** at the same hostname used in the signed URL; the default `localhost:9000` only works on the machine running the local stack. See the [system architecture](docs/architecture/system-overview.md) for service boundaries.

**Existing deployments:** If your database contains legacy inline/base64 PDF résumés, run [`infra/verify_and_migrate.sh`](infra/verify_and_migrate.sh) with the matching database and object-store configuration **before starting the updated API**. It verifies each migrated object before dropping the old field and stops on a missing or corrupt record. Back up your data first.

### Upgrading an existing database

Two upgrade paths apply the same end state:

- **Fresh database:** the API applies the consolidated schema in `server/api/src/schema.surql` once and records every known migration, through `016_application_dispatch_and_saved_job_index`, as its baseline. Do not pre-create tables or replay the numbered files first.
- **Versioned database:** the API reads `schema_migration` and applies only missing versions in order. An installation recorded through `014_cv_library` receives `015_saved_jobs_application_flow` and `016_application_dispatch_and_saved_job_index`; an installation recorded through `013` also receives `014`, whose preflight refuses to continue when an attached résumé upload has no matching `resume.source_upload_id`. `015` adds the saved-job flow without replacing an existing `saved_job` table. `016` adds `application_dispatch`, backfills pending dispatches for submitted applications that have an owned workspace but no existing dispatch or workflow run, restores only those pre-run `failed` workspaces to `pending`, and adds the saved-job keyset index. Orphaned submitted applications are reported in the API migration logs and are not given invented workspaces or dispatch rows.

Back up the database and object store before an upgrade, and verify the restored copy in a disposable environment. Never point `scripts/check-schema-parity.py` at a shared or production server: the checker creates and removes the `parity_schema` and `parity_migr` namespaces on the SurrealDB endpoint you pass it. Use a disposable instance for parity and upgrade rehearsal.

`application_dispatch` is the durable record between a committed application result and the intelligence service. It is created in the same transaction as the submitted application and is delivered after restart. The API claims due rows with a lease token, creates at most one idempotent workflow run per workspace, marks the row delivered with the run ID, and retries failed deliveries with bounded exponential backoff. A closed gRPC control connection is recreated on a later attempt. An operator does not need to replay the submission; restarting the API drains pending work on startup and then every five seconds.

## Native clients

Android, iOS, Linux, macOS, and Windows clients are in [`app/mobile`](app/mobile) and [`app/desktop`](app/desktop). On first launch, enter the **Rust API origin**, for example `https://api.example.org`—not the web app URL or an `/api/v1` path. Remote servers require HTTPS; loopback HTTP is for local development. `localhost` on a phone or another computer refers to that device, not your server. You can change servers from settings or the sign-in screen; switching signs out the current session.


| Platform                        | Source                                     | Build requirements                                                                |
| ------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------- |
| Android                         | [`app/mobile/android`](app/mobile/android) | JDK 17, Android SDK 34, Gradle                                                    |
| iOS                             | [`app/mobile/ios`](app/mobile/ios)         | Xcode on macOS                                                                    |
| Desktop (Linux, macOS, Windows) | [`app/desktop`](app/desktop)               | Rust, platform webview (WebKitGTK on Linux, WebView2 on Windows, WebKit on macOS) |


Native-client workflow runs upload GitHub Actions artifacts. Pushing a `v*` tag runs the release workflow and publishes packaged preview assets on a GitHub prerelease; the builds are unsigned (iOS is simulator-only).

Apple distribution builds require your own signing and provisioning. Native builds use generated API clients from [`contracts/native`](contracts/native).

### Browser extension

The Manifest V3 assistant in [`app/extension`](app/extension) scans visible LinkedIn and TopCV job listings on demand, scores them against your Mensetsu profile, and saves jobs for your review. Applications use a reviewed email draft when a usable employer address is present; otherwise, the employer page remains the fallback. You send or submit applications yourself, then confirm the outcome to queue interview preparation. The extension never sends email or clicks an employer’s final submit button. See Settings → Browser extension in Mensetsu for setup and workflow instructions.

Build both browser distributions with:

```bash
cd app/extension
npm run build
```

The build writes two complete, separately installable directories:

| Browser | Output                                   | Minimum version | Install                                                                                                                        |
| ------- | ---------------------------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Chrome  | `app/extension/dist/chrome`              | Chrome 116      | Open `chrome://extensions`, enable **Developer mode**, choose **Load unpacked**, and select that directory.                     |
| Firefox | `app/extension/dist/firefox`             | Firefox 128     | Open `about:debugging#/runtime/this-firefox`, choose **Load Temporary Add-on**, and select `app/extension/dist/firefox/manifest.json`. |

Chrome 116 or newer and Firefox 128 or newer are required. Firefox runs the background code as a module event page (`background.scripts`); Chrome runs the same shared source as a module service worker (`background.service_worker`). Both builds request extension storage (`storage.session` is used for tokens; Firefox 128 supports it) and `activeTab`, not broad job-site host access.

On first run the popup asks for the **API base URL**. Enter the Rust API origin only, for example `https://api.example.org`—not the web app URL and not an `/api/v1` path. The extension rejects credentials, paths, queries, and fragments, requires HTTPS, and allows plain HTTP only for loopback development (`localhost`, `*.localhost`, or `127.0.0.1`) when you explicitly enable it. Approve the host-permission prompt for that origin. The extension then requests the unauthenticated `GET /health` endpoint with a timeout and requires a healthy response before it saves the origin and shows its sign-in form. If the probe fails, the origin is not saved and the sign-in form is not shown. Sign-in supports TOTP two-factor challenge.

Change the API base URL later from the popup's settings (gear) button. Changing to a different origin clears the extension's access, refresh, and temporary 2FA tokens before the new origin is saved, so an existing token is never sent to a new server.

Saved jobs load in pages of 25. **Load more** appends the next page and disappears when no cursor remains; **Refresh** returns to the first page. A saved job can be awaiting approval, approved, dismissed, or linked to an application. For a linked application, the popup shows **Submitted/queued** while interview preparation is waiting to be delivered, **Processing** while the workspace runs, **Ready** when preparation is complete, **Failed** when the workspace failed, and **Continue** for other linked states. Interview preparation is queued only after you confirm that you submitted the employer application; the extension and API never send employer email or click the employer's final submit action. Re-scanning the same job with new details moves a nonterminal linked application to **Needs review**, so you can approve the revised facts before preparing again.

## Architecture


| Component                                        | Responsibility                                                             |
| ------------------------------------------------ | -------------------------------------------------------------------------- |
| [`app/landing`](app/landing)                     | Astro public landing & documentation; candidate onboarding and guides      |
| [`app/web`](app/web)                             | Nuxt web UI; proxies API requests and manages browser sessions             |
| [`server/api`](server/api)                       | Rust API; identity, authorization, workspaces, storage access, and OpenAPI |
| [`server/intelligence`](server/intelligence)     | Python intelligence service and worker; research and analysis              |
| [`contracts`](contracts)                         | REST/OpenAPI and internal gRPC contracts; generated client code            |
| SurrealDB, Redis, SearXNG, S3-compatible storage | Workspace data, job coordination, web search, and private PDF objects      |


The [architecture overview](docs/architecture/system-overview.md) covers service communication and workflow execution.

## Development and verification

Local development requires Rust, Bun, Python 3.12+, `uv`, and Docker Compose for supporting services. Copy and configure `.env` as above. `make infra-up` starts SurrealDB and Redis; research and uploads additionally need SearXNG and MinIO (`docker compose up -d searxng minio minio-init`). Start the applications in separate terminals with `make dev-api`, `make dev-ai`, `make dev-worker`, `make dev-web`, and `make dev-landing`.

The Rust API applies the ordered SQL files in `infra/migrations/` during startup and records completed versions in `schema_migration`. On first startup it applies the current consolidated schema once and records the existing migrations as its baseline; later-added migrations run once when their version is absent. Migration errors stop API startup. See the [schema inventory](docs/architecture/schema-inventory.md) for current table and optional-field evidence; it is advisory and does not authorize destructive cleanup.

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