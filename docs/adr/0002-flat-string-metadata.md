# 0002. Front matter flattens to `HashMap<String, String>` with dotted keys

- **Status:** accepted
- **Date:** 2024-10-03 (recorded 2026-09-06)

## Context

Consumers (`staticdatagen`, `ssg`) feed front matter into templates and
`<meta>` generation, both of which want strings. YAML, TOML and JSON
front matter differ in type systems and nesting; a typed tree would push
every consumer to handle three shapes.

## Decision

`Metadata` is a flat `HashMap<String, String>`. Nested tables join with
`.` (`author.name`), sequences render as `[a, b]`, and every other
scalar takes its `Display` form. The three flatteners are kept
structurally parallel so a change to one is visibly missing from the
others.

## Consequences

- One shape for consumers regardless of front-matter format.
- Type information is lost at the boundary: `3` and `"3"` are the same
  string. A typed extraction API can be added beside this one; it does
  not replace it.
- Element-level access to arrays of objects is out of scope for the
  flat map by design.
