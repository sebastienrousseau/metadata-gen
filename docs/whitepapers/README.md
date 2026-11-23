<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Engineering write-ups

Longer explanations of decisions in `metadata-gen`, written from the
repository: its code, tests, changelog and decision records. Every
number in them names where it came from; where something was not
measured, the paper says so.

| Paper | Date | Summary |
| :--- | :--- | :--- |
| [One map is not enough: the shapes of front matter](01-shapes-of-front-matter.md) | 2026-11-23 | Why the flat `HashMap<String, String>` stayed, and the typed, borrowed and streaming APIs that grew beside it. |
| [Front matter at the edge: a WASI 0.2 component](02-front-matter-at-the-edge.md) | 2026-11-23 | Building the crate as a component, proving it returns what native code returns, and running it in a Cloudflare Worker. |
| [Property-testing a front-matter parser](03-property-testing-front-matter.md) | 2026-11-23 | The round-trip and involution properties, the bug the fuzzer found, and how CI scales the case count. |

These are drafts for the project's own documentation. Posting them
elsewhere (a blog, r/rust, lobste.rs, Hacker News) is a maintainer
decision and has not been done.
