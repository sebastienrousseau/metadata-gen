<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Fermyon Spin

Spin runs WASI 0.2 components and can plug one component into another
at load time: a *component dependency* in `spin.toml`. The sample's
HTTP handler,
[`examples/deploy/http-handler/`](../../examples/deploy/http-handler/),
imports the `metadata-gen:lib/metadata` interface, and Spin satisfies
that import with the component built from [`wasi/`](../../wasi/). The
app is in [`examples/deploy/spin/`](../../examples/deploy/spin/).

**What was checked (2026-11-26):** the steps below on macOS with Rust
1.100 and Spin 4.3.0. `spin up` served the app and answered every
request in step 4. CI runs `spin build`, `spin up` and
[`examples/deploy/smoke.sh`](../../examples/deploy/smoke.sh) on every
push (the `deploy sample (Spin)` job in `quality.yml`). It was **not**
deployed to Fermyon Cloud, so step 5 is untested.

**Measured: 30 s on an Apple A18 Pro Mac (6 cores, macOS), 2026-11-26**, from `git clone` to the first `200` from
`spin up`, with cold `target/` directories and a warm Cargo registry.
Installing Spin is not included.

## 1. Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- [Spin](https://spinframework.dev/) 3.0 or newer (component
  dependencies arrived in 3.0; checked with 4.3.0)
- A Fermyon Cloud account, for the deploy step only

## 2. Build

```bash
git clone https://github.com/sebastienrousseau/metadata-gen.git
cd metadata-gen/examples/deploy/spin
spin build   # cargo builds wasi/ and ../http-handler for wasm32-wasip2
```

## 3. How it fits together

[`spin.toml`](../../examples/deploy/spin/spin.toml) routes every path to
the `frontmatter` component, whose source is the HTTP handler, and
declares the dependency:

```toml
[component.frontmatter.dependencies]
"metadata-gen:lib/metadata@0.0.10" = { path = "../../../wasi/target/wasm32-wasip2/release/metadata_gen_component.wasm" }
```

The handler holds no parser: it reads the body, calls
`metadata::extract`, and writes the result as JSON. `POST` returns the
front matter as a JSON object, or HTTP 422 with `{"error": ...}`; any
other method gets 405.

## 4. Run it locally

```bash
spin up
# in another terminal:
curl -X POST --data-binary $'---\ntitle: Hi\nauthor:\n  name: Ada\n---\nbody' http://127.0.0.1:3000/
# {"author.name":"Ada","title":"Hi"}
curl -X POST --data-binary 'plain' http://127.0.0.1:3000/
# {"error":"Failed to extract metadata: No valid front matter found."}  (HTTP 422)
../smoke.sh http://127.0.0.1:3000/   # the same checks CI runs
```

## 5. Deploy to Fermyon Cloud

```bash
spin plugins install cloud
spin cloud login
spin cloud deploy
```

Not run for this guide; see the note at the top. If the platform
does not accept component dependencies, deploy the composed component
the wasmCloud sample builds instead (`examples/deploy/wasmcloud/build.sh`
writes it to `build/frontmatter.wasm`): point `source` at that file
and remove the `dependencies` table.
