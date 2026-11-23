<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Canonical encoding of metadata, version 1

- **Status:** stable from 0.0.10
- **Implemented by:** `metadata_gen::sign::canonical_bytes` (feature `pqc-sign`)
- **Decision record:** [ADR-0010](../adr/0010-pqc-canonical-encoding.md)

The canonical encoding turns a flat metadata map (string keys, string
values, as `extract_metadata` produces) into one byte string. Two maps
encode to the same bytes exactly when they hold the same set of
`(key, value)` pairs. It is the input to the ML-DSA signature, so it
must not depend on the platform, the hash-map iteration order, or the
front-matter format the map came from.

## Encoding

All integers are unsigned 64-bit, big-endian (`u64be`).

```text
canonical = magic count entry*
magic     = "MDGCAN01"                 ; 8 ASCII bytes: 4D 44 47 43 41 4E 30 31
count     = u64be                      ; number of entries
entry     = key-len key value-len value
key-len   = u64be                      ; length of key in bytes
key       = UTF-8 bytes of the key
value-len = u64be                      ; length of value in bytes
value     = UTF-8 bytes of the value
```

Entries appear in ascending order of their key's UTF-8 bytes,
compared byte by byte, a shorter key sorting before a longer key it is
a prefix of. Keys are unique, so the order is total.

Keys and values are encoded exactly as stored. There is no Unicode
normalisation, case folding, whitespace trimming, or number
formatting at this stage; those decisions belong to the extraction
step (ADR-0002 and the README's output-stability guarantee), and the
signature covers their result.

Length prefixes on both fields mean no two different maps share an
encoding: `{"ab": "c"}` and `{"a": "bc"}` differ.

## Example

The map `{"b": "xy", "a": ""}` encodes as (hex, grouped by field):

```text
4D44474341 4E3031          magic "MDGCAN01"
0000000000000002           count = 2
0000000000000001 61        key "a"
0000000000000000           value ""
0000000000000001 62        key "b"
0000000000000002 7879      value "xy"
```

`tests/test_sign.rs` (`canonical_encoding_is_exact`) asserts these
bytes.

## Signature

`Metadata::sign` signs the encoding with ML-DSA-65 (FIPS 204), using
the deterministic variant of `ML-DSA.Sign` and the context string
`metadata-gen/canonical/v1` (25 ASCII bytes). `Metadata::verify`
recomputes the encoding and checks the signature under the same
context. A signature is 3,309 bytes and a public key 1,952 bytes, as
FIPS 204 specifies for ML-DSA-65.

## Versioning

A change to anything above is a new version: a new magic (`MDGCAN02`)
and a new context string (`metadata-gen/canonical/v2`), recorded in a
new ADR. A signature made under one version never verifies under
another, because the context string differs.
