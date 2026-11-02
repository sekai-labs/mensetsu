# Mensetsu (面接)

Mensetsu helps you prepare for a *specific* technical interview. Give it a résumé, a job description, and a target company; it researches the company with citations, shows where your experience matches the role, maps what you need to learn, and builds practice questions. Research runs in the background, so you can return to a workspace as its evidence and study plan take shape.

The [web app](app/web) is built with Nuxt. A [Rust API](server/api) owns accounts, workspaces, permissions, storage, and the OpenAPI contract. A [Python service and worker](server/intelligence) handle research and analysis. SurrealDB stores workspace and graph data, Redis coordinates jobs and web sessions, and private S3-compatible storage holds uploaded PDFs. Clients never receive the internal service token or a raw web-session access token.

You can also use native clients for [Android and iOS](app/mobile) or [Linux, macOS, and Windows](app/desktop). They use the Rust API directly; there is no Flutter client.

## Run it locally

If you already have a Mensetsu database with inline PDF résumés, follow [the migration steps](#private-pdf-r%C3%A9sum%C3%A9s) **before starting the new API**.

Docker Compose is the shortest path. Copy the template, set strong values for `JWT_SECRET`, `ENCRYPTION_KEY`, `INTERNAL_SERVICE_TOKEN`, and `SEARXNG_SECRET` in `.env`, then start the stack:

```bash
cp .env.example .env
docker compose up -d --build
```

Open the web app at **http://localhost:3001**. The API health endpoint is **http://localhost:3000/health/live**; SurrealDB listens on port 8000, and the local MinIO API and console listen on ports 9000 and 9001. Do not expose the sample database or storage credentials outside local development. Stop the stack with `docker compose down` (add `-v` only if you deliberately want to delete its data).

For local development, you will need Rust, Bun, Python 3.12+, and `uv`. `make infra-up` starts SurrealDB and Redis. The API, web app, intelligence gRPC service, and worker run separately with `make dev-api`, `make dev-web`, `make dev-ai`, and `make dev-worker`. Research and PDF uploads also need SearXNG and MinIO; `docker compose up -d searxng minio minio-init` starts those services against the same `.env`.

## Contracts and checks

Rust exports the public REST/OpenAPI schema; protobuf files in [`contracts/grpc/proto/v1`](contracts/grpc/proto/v1) define service-to-service gRPC. TypeScript and the Kotlin, Swift, Rust, and C# native clients are generated from those sources. After changing an exposed API, run `make contracts`, then `make contracts-check`. Generation uses Docker and OpenAPI Generator 7.15.0. `make schema-parity` checks the SurrealDB migrations against the current schema and requires a running database.

Run the application tests with `make test`, or use `make test-api`, `make test-ai`, and `make test-web` for one service. `make check` also checks contracts, schema parity, and source linting.

**Browser end-to-end tests erase the configured SurrealDB database and flush Redis.** Use only a disposable test environment. Inspect the reset without changing data, or run the suite, respectively:

```bash
RESET_DRY_RUN=1 MENSETSU_ALLOW_RESET=1 make e2e-reset
MENSETSU_ALLOW_RESET=1 make e2e
```

## Native builds

Each client has its own toolchain:

| Platform | Source | Build requirement |
| --- | --- | --- |
| Android | `app/mobile/android` | JDK 17, Android SDK 34; Gradle builds a debug APK |
| iOS | `app/mobile/ios` | Xcode on macOS; an unsigned simulator app can be built without signing credentials |
| Linux | `app/desktop/linux` | Rust, GTK4, libsecret; `cargo build --manifest-path app/desktop/linux/Cargo.toml` |
| macOS | `app/desktop/macos` | Xcode on macOS; unsigned `.app` archive |
| Windows | `app/desktop/windows` | .NET and Windows SDK; `dotnet build app/desktop/windows/Mensetsu.sln -p:Platform=x64` |

Generated API clients live in `contracts/native`; the Android Gradle module at `app/mobile/android/contracts` compiles the generated Kotlin sources without duplicating them. Keep `Cargo.lock`, Xcode projects and schemes, and Visual Studio projects: they are build inputs. Cargo `target/` and .NET `bin/`/`obj/` are disposable outputs.

The GitHub-only native workflow builds and uploads artifacts for all five platforms. To also run real UI smoke tests, set the GitHub repository variable `NATIVE_SMOKE_API_URL` to a test Rust API accessible from hosted runners. Without it, the UI tests are skipped; the builds still run. iOS and macOS archives are unsigned; distributing installable Apple builds requires Apple signing and provisioning outside this workflow.

## Private PDF résumés

PDFs go directly from the client to a private S3-compatible bucket through a short-lived signed `PUT`. Rust authorizes the upload and checks the size, type, and PDF header before attaching it to a workspace. The Python worker obtains a short-lived signed download through authenticated internal gRPC when it needs to extract text. Workspace JSON does not carry PDF bytes or base64. Local MinIO's public signing endpoint must be reachable by the browser or device at the *exact* hostname in the signed URL; set `S3_ENDPOINT_PUBLIC` to a shared HTTPS hostname for remote devices.

**Migrating existing data:** If your database has older inline/base64 PDF records, start SurrealDB and the private bucket, configure the matching host-side S3 and SurrealDB environment, and run `bash infra/verify_and_migrate.sh` **before starting the new API**. It copies PDFs to private storage, verifies the objects, and only drops the old field after every record is accounted for. On a corrupt or missing record it stops without dropping data; fix that record and rerun. The API refuses to start while legacy PDF rows remain.

## GitHub builds

Codeberg is the source repository; your existing synchronization brings code to GitHub. GitHub alone holds `.github/workflows/` and publishes the `api`, `intelligence`, `worker`, and `web` images to `ghcr.io`. Native builds upload an APK, an iOS simulator app, a Linux binary, an unsigned macOS app, and a Windows executable bundle as GitHub Actions artifacts. Pull requests build images without publishing. Keep the GitHub-only workflows out of Codeberg and ensure the existing sync preserves them: a force-push mirror over GitHub `main` would overwrite them.

Licensed under [Apache-2.0](LICENSE).
