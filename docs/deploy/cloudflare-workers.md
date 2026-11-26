<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Cloudflare Workers

Workers run JavaScript and core WebAssembly modules, not components,
so the component is transpiled with
[`jco`](https://github.com/bytecodealliance/jco) into core modules plus
JavaScript glue, and the Worker instantiates that. The sample is in
[`examples/deploy/cloudflare-workers/`](../../examples/deploy/cloudflare-workers/).

**What was checked (2026-11-23):** the steps below, run from a clean
checkout on macOS with Rust 1.100, Node 26, jco 1.42.0 and wrangler
4.162.0. `wrangler dev` served the Worker locally on workerd and answered
both requests shown. `wrangler deploy --dry-run` bundled it to 998 KiB
(286 KiB gzipped).

**Deployed (2026-11-26):** step 5, run from a fresh clone of `main` at
e27ecaa with wrangler 4.162.0, uploaded 1011.64 KiB (290.50 KiB
gzipped) to a Cloudflare account; the Worker reported a 1 ms startup
time. [`examples/deploy/smoke.sh`](../../examples/deploy/smoke.sh)
against its `workers.dev` URL got the documented `200`, `422` and
`405`.

**Measured: 2 min (123 s) on an Apple A18 Pro Mac (6 cores, macOS),
2026-11-25**, from `git clone` to the first `200` from `wrangler dev`:
clone, `npm ci`, `npm run build` with a cold `wasi/target`, then the
first `curl` in step 4. The clone was from a local copy and the Cargo
registry and npm caches were warm, so a first run on a new machine
also pays for those downloads. The deploy step is not included.

## 1. Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- Node.js 20 or newer and npm
- A Cloudflare account, for the deploy step only

## 2. Build and transpile

```bash
git clone https://github.com/sebastienrousseau/metadata-gen.git
cd metadata-gen/examples/deploy/cloudflare-workers
npm ci          # exact versions from package-lock.json
npm run build   # cargo build of wasi/, then jco transpile into src/gen/
npx --no-install wrangler deploy --dry-run --outdir dist   # bundle only
```

`package.json` overrides `minimatch` to 11.0.6: jco 1.42.0 pulls
10.2.6 through `@bytecodealliance/jco-std`, which GHSA-g3hv-75wg-m249
affects. CI runs these three commands and `npm audit` on every push
(the `deploy sample (Cloudflare Workers)` job in `quality.yml`).

`build.sh` runs `jco transpile ... --instantiation async`, which emits
`src/gen/metadata-gen.js` plus three core modules. Use the locally
installed jco (`npx --no-install jco`, as the script does): a bare
`npx jco` fetches an unrelated npm package named `jco`.

## 3. The Worker

[`src/index.js`](../../examples/deploy/cloudflare-workers/src/index.js)
imports the three `.wasm` files (wrangler compiles them into
`WebAssembly.Module`s), instantiates the component once with the WASI
shims from `@bytecodealliance/preview2-shim`, and answers `POST /` with
the front matter as a JSON object, or HTTP 422 with the error text.

## 4. Run it locally

```bash
npm run dev
# in another terminal:
curl -X POST --data-binary $'---\ntitle: Hello from a Worker\nauthor:\n  name: Ada\n---\nbody' http://localhost:8787/
# {"author.name":"Ada","title":"Hello from a Worker"}
curl -X POST --data-binary 'plain' http://localhost:8787/
# {"error":"Failed to extract metadata: No valid front matter found."}  (HTTP 422)
```

## 5. Deploy

```bash
npx wrangler login
npm run deploy
```

`wrangler deploy` prints the Worker's `workers.dev` URL; check it with
`../smoke.sh https://<worker>.<subdomain>.workers.dev/`. The bundle is
within the Workers Free plan's 3 MiB compressed limit.

## Limits

- The component's WASI imports (clocks, random seed, stdio) are served
  by the browser build of `preview2-shim`; the extraction code does not
  use the filesystem or network.
- Every isolate instantiates the component on its first request.
