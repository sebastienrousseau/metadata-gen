# 0003. `<meta>` extraction is a streaming `quick-xml` pass, not a DOM

- **Status:** accepted
- **Date:** 2026-06-28 (recorded 2026-09-06)

## Context

`extract_meta_tags` used to build a full document with `scraper`
(html5ever + selectors). That cost a DOM allocation for every page for
the sake of one element name, and pulled a large dependency tree into
every consumer.

## Decision

Extraction is a single forward pass with `quick-xml`'s reader. Only
`Start` and `Empty` events named `meta` are inspected; attribute names
compare case-insensitively; entity references in values are decoded.
A reader error ends the scan and returns what was found so far.

## Consequences

- Linear time, no DOM, one small dependency.
- Tolerance is deliberate: the usual input is a whole HTML page, which a
  strict XML reader will not accept end to end. A `<meta>` tag after the
  first unrecoverable error is not found. That trade is documented in
  the function's docs and pinned by tests.
- quick-xml's API moves (0.42 changed attribute values to `str`); the
  collector is the only place that needs to follow.
