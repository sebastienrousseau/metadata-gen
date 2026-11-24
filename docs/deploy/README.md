<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Deploying the WASI component

`metadata-gen` builds as a WASI 0.2 component
([`wasi/README.md`](../../wasi/README.md), ADR-0009) that exports
`extract`, `extract-meta-tags` and `escape-html`. These guides put it
behind an HTTP endpoint on three hosts. Each guide says what was run to
check it and what was not.

| Host | Guide | Checked |
| :--- | :--- | :--- |
| Cloudflare Workers | [cloudflare-workers.md](cloudflare-workers.md) | Built and served locally with `wrangler dev` (workerd); `wrangler deploy --dry-run` bundles it. Not deployed to a Cloudflare account. |
| Fermyon Spin | [spin.md](spin.md) | Not run. The steps follow Spin's documented component-dependency support. |
| wasmCloud | [wasmcloud.md](wasmcloud.md) | Not run. The steps follow wasmCloud's documented composition workflow. |

The component itself is checked in CI on every push: it validates with
`wasm-tools`, stays under 1 MiB, and returns byte-for-byte what the
native crate returns under `wasmtime` (`python3 -I wasi/check.py`).
