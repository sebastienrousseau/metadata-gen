<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# wasmCloud

wasmCloud runs WASI 0.2 components. The sample composes the component
built from [`wasi/`](../../wasi/) into the HTTP handler in
[`examples/deploy/http-handler/`](../../examples/deploy/http-handler/)
before it runs, with [`wac`](https://github.com/bytecodealliance/wac),
so the host sees one component that exports `wasi:http` and needs
nothing from metadata-gen. The app is in
[`examples/deploy/wasmcloud/`](../../examples/deploy/wasmcloud/).

**What was checked (2026-11-26):** the steps below on macOS with Rust
1.100, wash 2.12.1 and wac 0.12.0. `wash dev` served the
component and answered every request in step 4. CI runs `wash build`,
`wash dev` and [`examples/deploy/smoke.sh`](../../examples/deploy/smoke.sh)
on every push (the `deploy sample (wasmCloud)` job in `quality.yml`).
It was **not** deployed to a wasmCloud cluster, so step 5 is untested.

**Measured: 33 s on an Apple A18 Pro Mac (6 cores, macOS), 2026-11-26**, from `git clone` to the first `200` from
`wash dev`, with cold `target/` directories and a warm Cargo registry.
Installing wash and wac is not included.

## 1. Prerequisites

- Rust with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- [wash](https://wasmcloud.com/docs/installation) 2.x (checked with 2.12.1)
- wac: `cargo install --locked wac-cli`
- For the deploy step: a Kubernetes cluster running the wasmCloud
  runtime operator, and an OCI registry you can push to

## 2. Build

```bash
git clone https://github.com/sebastienrousseau/metadata-gen.git
cd metadata-gen/examples/deploy/wasmcloud
wash build
```

`wash build` runs the command in
[`.wash/config.yaml`](../../examples/deploy/wasmcloud/.wash/config.yaml),
[`build.sh`](../../examples/deploy/wasmcloud/build.sh): it builds both
components for `wasm32-wasip2` and runs

```bash
wac plug http-handler.wasm --plug metadata_gen_component.wasm -o build/frontmatter.wasm
```

## 3. The handler

The handler reads the request body, calls `metadata::extract`, and
writes the result as JSON. `POST` returns the front matter as a JSON
object, or HTTP 422 with `{"error": ...}`; any other method gets 405.

## 4. Run it locally

```bash
wash dev
# in another terminal:
curl -X POST --data-binary $'---\ntitle: Hi\nauthor:\n  name: Ada\n---\nbody' http://127.0.0.1:8000/
# {"author.name":"Ada","title":"Hi"}
curl -X POST --data-binary 'plain' http://127.0.0.1:8000/
# {"error":"Failed to extract metadata: No valid front matter found."}  (HTTP 422)
../smoke.sh http://127.0.0.1:8000/   # the same checks CI runs
```

## 5. Deploy

wasmCloud 2 runs workloads on Kubernetes through its runtime operator.
Push the component to a registry, then apply a `WorkloadDeployment`
that names it:

```bash
wash oci push ghcr.io/<you>/metadata-gen-frontmatter:0.0.10 build/frontmatter.wasm
```

Not run for this guide; see the note at the top. The cluster setup is
in wasmCloud's documentation.
