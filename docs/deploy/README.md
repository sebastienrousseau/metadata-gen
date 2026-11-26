<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Deploying the WASI component

`metadata-gen` builds as a WASI 0.2 component
([`wasi/README.md`](../../wasi/README.md), ADR-0009) that exports
`extract`, `extract-meta-tags` and `escape-html`. These guides put it
behind an HTTP endpoint on three hosts. Each guide says what was run to
check it and what was not.

| Host | Guide | Checked |
| :--- | :--- | :--- |
| Cloudflare Workers | [cloudflare-workers.md](cloudflare-workers.md) | Built and served locally with `wrangler dev` (workerd); deployed to a Cloudflare account on 2026-11-26 and checked with `smoke.sh` at its `workers.dev` URL. |
| Fermyon Spin | [spin.md](spin.md) | Served locally with `spin up` and checked in CI on every push. Not deployed to Fermyon Cloud. |
| wasmCloud | [wasmcloud.md](wasmcloud.md) | Served locally with `wash dev` and checked in CI on every push. Not deployed to a wasmCloud cluster. |

The component itself is checked in CI on every push: it validates with
`wasm-tools`, stays under 1 MiB, and returns byte-for-byte what the
native crate returns under `wasmtime` (`python3 -I wasi/check.py`).
All three samples answer the same contract, which
[`examples/deploy/smoke.sh`](../../examples/deploy/smoke.sh) checks
against a running server: `POST` a document and get its front matter
as JSON, `422` with `{"error": ...}` when it has none, `405` for any
other method.
