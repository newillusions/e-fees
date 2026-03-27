---
name: deploy-containers
description: Use when deploying e-fees-api or e-fees-scope containers after code changes. Covers build, push to Forgejo registry, and Unraid container update. Also use when asked about container deployment, image builds, or registry setup.
---

# Deploy E-Fees Containers

Build and push e-fees-api and e-fees-scope Docker images to the Forgejo container registry, then update running containers on Unraid AI server.

## Architecture

```
Code (fee-prop repo) → Build on AI server → Push to Forgejo registry → Unraid UI pulls update
```

| Component | Registry Image | Container IP | Port |
|-----------|---------------|-------------|------|
| e-fees-api | `forge.mms.name/emittiv/e-fees-api:latest` | 10.0.21.80 | 3200 |
| e-fees-scope | `forge.mms.name/emittiv/e-fees-scope:latest` | 10.0.21.81 | 3201 |

## Quick Deploy (Both Containers)

```bash
# SSH to AI server
ssh unraid-ai

# Pull latest source
cd /mnt/user/appdata/e-fees-api/source && git pull origin main

# Build and push API
docker build -t forge.mms.name/emittiv/e-fees-api:latest -f e-fees-api/Dockerfile .
docker push forge.mms.name/emittiv/e-fees-api:latest

# Build and push Scope
docker build -t forge.mms.name/emittiv/e-fees-scope:latest -f e-fees-scope/Dockerfile .
docker push forge.mms.name/emittiv/e-fees-scope:latest
```

Then in Unraid UI: Docker tab → Check for Updates → Update for each container.

## Deploy Single Container

```bash
ssh unraid-ai
cd /mnt/user/appdata/e-fees-api/source && git pull origin main

# API only
docker build -t forge.mms.name/emittiv/e-fees-api:latest -f e-fees-api/Dockerfile .
docker push forge.mms.name/emittiv/e-fees-api:latest

# OR Scope only
docker build -t forge.mms.name/emittiv/e-fees-scope:latest -f e-fees-scope/Dockerfile .
docker push forge.mms.name/emittiv/e-fees-scope:latest
```

## First-Time Setup

### Registry Login (once per server)

```bash
ssh unraid-ai "docker login forge.mms.name"
# Username: martin
# Password: Forgejo access token with package:write scope
```

### Template Configuration

XML templates live in `forge.mms.name/emittiv/docker-templates` repo under `martin/`:
- `e-fees-api.xml` — Repository: `forge.mms.name/emittiv/e-fees-api:latest`
- `e-fees-scope.xml` — Repository: `forge.mms.name/emittiv/e-fees-scope:latest`

Unraid reads these templates to know where to pull images from.

## Key Details

| Item | Value |
|------|-------|
| AI server SSH | `ssh unraid-ai` (10.0.20.11, root) |
| Source on server | `/mnt/user/appdata/e-fees-api/source/` |
| Forgejo repo | `forge.mms.name/emittiv/fee-prop` |
| Registry | `forge.mms.name/emittiv/-/packages/container/` |
| Template repo | `forge.mms.name/emittiv/docker-templates` |
| Build time | ~5-10 min per container (Rust compilation) |

## Environment Variables

### e-fees-api
`SURREAL_URL`, `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`, `API_KEY`, `API_PORT`, `NC_SSH_HOST`, `NC_SSH_USER`, `NC_SSH_KEY_PATH`, `NC_SCRIPT_PATH`, `NC_OWNER` (default: 99:100)

### e-fees-scope
`SURREAL_URL`, `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`, `API_KEY`, `API_PORT`, `OLLAMA_URL`

## Critical Rules

- **NEVER `docker restart`** — reuses old image. Always update from Unraid UI or stop→rm→create.
- **NEVER `docker rm`/`docker stop`** without user confirmation — use Unraid UI instead.
- Both builds run in parallel (independent Dockerfiles, no shared layers).
- Source repo is a monorepo — both Dockerfiles are in the same git repo.
- `container-utils` crate is fetched from Forgejo during build (git dependency in Cargo.toml).
