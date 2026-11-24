<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# 0006. `<meta>` extraction scans with `memchr` and parses tags with `oxml`

- **Status:** accepted (supersedes [ADR-0003](0003-streaming-meta-extraction.md))
- **Date:** 2026-11-01 (recorded 2026-11-23)

## Context

[ADR-0003](0003-streaming-meta-extraction.md) recorded `<meta>`
extraction as a single forward pass of `quick-xml`'s reader. That
decision held from 0.0.5 to 0.0.8. Two forces moved it:

- `quick-xml` was the last third-party markup parser in the graph, and
  its API moved between minor releases (0.42 changed attribute values
  to `str`), so every bump touched the collector.
- `oxml` is a first-party, `#![forbid(unsafe_code)]` crate from the
  same maintainer that parses attributes and decodes XML and HTML
  character and entity references, which is all the collector needs.

The alternatives were to keep `quick-xml`, or to return to a full HTML5
parser (`scraper`, `html5ever`), which ADR-0003 already rejected for
the DOM it allocates per page.

## Decision

`extract_meta_tags` and `extract_meta_tags_lenient` (feature `html`)
find each `<` with `memchr`, find the end of the tag, and hand only
tags named `meta` to `oxml` for attribute parsing and reference
decoding. `oxml` is pinned exactly (`=0.0.13`), like every first-party
0.0.x crate ([ADR-0004](0004-first-party-exact-pins.md)).

The observable behaviour of ADR-0003 is kept: one forward pass, no
DOM, attribute names compared case-insensitively, `name`, `property`
and `http-equiv` identify a tag in that order, and a scanner error ends
the scan and returns what was found so far.

## Consequences

- `quick-xml` is gone from the dependency graph (0.0.9 changelog).
- An `oxml` release reaches consumers only through a `metadata-gen`
  release that moves the pin, the same cost ADR-0004 accepts for
  `noyalib`.
- The tolerance trade from ADR-0003 is unchanged: a `<meta>` tag after
  the first unrecoverable error is not found, and
  `extract_meta_tags_lenient` reports where the scan stopped.
