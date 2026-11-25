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
(286 KiB gzipped). It was **not** deployed to a Cloudflare account, so
the deploy step and the production response are untested.

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

Not run for this guide; see the note at the top. The bundle is within
the Workers Free plan's 3 MiB compressed limit.

## Limits

- The component's WASI imports (clocks, random seed, stdio) are served
  by the browser build of `preview2-shim`; the extraction code does not
  use the filesystem or network.
- Every isolate instantiates the component on its first request.
