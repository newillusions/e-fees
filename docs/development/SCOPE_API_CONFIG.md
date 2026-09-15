# Scope service API config (`VITE_SCOPE_API_URL` / `VITE_SCOPE_API_KEY`)

`src/lib/api/scope.ts` calls the e-fees-scope microservice directly via
`fetch()` from inside the desktop app's webview (this is the one frontend
module that talks HTTP directly rather than going through a Tauri command -
everything else uses `invoke()`, which stays server-side and never bakes a
secret into the shipped bundle).

## Why there is no fallback in code

Vite's `import.meta.env.VITE_*` values are resolved and inlined into the
JavaScript bundle **at build time**, not read at runtime. Until this PR,
`scope.ts` had a hardcoded fallback (`efees-scope-2026-s7k2m9xp` and
`http://10.0.21.81:3201`) used whenever the env vars were unset - and no build
in this repo (`.github/workflows/build-releases.yml`, the only workflow that
runs `npm run tauri:build`) ever set them. That means the fallback was not a
"just in case" value; it was the *only* value any shipped release ever
carried, and it ended up committed to this repo's public GitHub mirror
(`newillusions/e-fees`) in plaintext.

The fix removes the fallback. `scopeRequest()` now throws a clear error at
call time if either var is unset, instead of silently working with a baked
default.

## Local development

Run the app with the values injected into your shell, via the credential
wrapper:

```bash
~/.claude/scripts/creds-run.sh <VAR_NAME_FOR_THE_SCOPE_KEY> -- npm run tauri:dev
```

**Open item, flagged rather than guessed:** there is currently no entry in
`~/.claude/credential-directory.md` for the e-fees-scope service's `API_KEY`
(the value configured on the deployed container, distinct from
`EFEES_API_KEY`). Until one is added, set the two vars directly in your shell
for local dev:

```bash
export VITE_SCOPE_API_URL="http://10.0.21.81:3201"
export VITE_SCOPE_API_KEY="<the e-fees-scope container's API_KEY>"
npm run tauri:dev
```

Never commit a `.env` file containing the real key. `vite.config.ts` already
scopes env-var pickup to the `VITE_`/`TAURI_` prefixes via `envPrefix`.

## CI / release builds

`.github/workflows/build-releases.yml` (GitHub Actions - this is a
tag-triggered release build, not the Forgejo PR/push CI in `.forgejo/workflows/`)
must supply both vars as GitHub Actions repository secrets before the
`Build Tauri app` step, the same way it already supplies
`TAURI_SIGNING_PRIVATE_KEY`. See that workflow's `Build Tauri app` steps.

## Tests

`vitest.config.ts` sets `test.env.VITE_SCOPE_API_URL` /
`VITE_SCOPE_API_KEY` to dummy values so `scope.test.ts` (which mocks
`fetch()` and never hits the network) doesn't need to configure anything
itself.
