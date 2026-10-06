#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Blocking release preflight. Run it on a local, signed, annotated tag
# BEFORE pushing the tag; the release workflow re-checks what it can, but
# only this script can see the signature and the local working tree.
#
#   scripts/release-preflight.sh v0.0.8
#
# Proves: the tag exists, is annotated and signed, points at HEAD, its
# subject is "metadata-gen vX.Y.Z"; every version-bearing file agrees;
# docs/releases/vX.Y.Z.md carries the Highlights; the packaged archive
# holds the current manifest, README and CHANGELOG; and the gate is green.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

tag="${1:-}"
[ -n "$tag" ] || { echo "usage: $0 vX.Y.Z" >&2; exit 2; }
version="${tag#v}"
fail() { echo "preflight: $*" >&2; exit 1; }

echo "== tag"
git rev-parse -q --verify "refs/tags/$tag" >/dev/null || fail "tag $tag does not exist locally"
[ "$(git cat-file -t "refs/tags/$tag")" = "tag" ] || fail "$tag is lightweight; create a signed annotated tag"
[ "$(git rev-parse "$tag^{commit}")" = "$(git rev-parse HEAD)" ] || fail "$tag does not point at HEAD"
subject="$(git tag -l --format='%(contents:subject)' "$tag")"
[ "$subject" = "metadata-gen v$version" ] || fail "tag subject is '$subject', expected 'metadata-gen v$version'"
git tag -v "$tag" >/dev/null 2>&1 || fail "$tag has no valid signature"
echo "   annotated, signed, subject ok, points at HEAD"

echo "== working tree"
[ -z "$(git status --porcelain)" ] || fail "working tree is not clean"

echo "== versions"
manifest="$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)"
[ "$manifest" = "$version" ] || fail "Cargo.toml says $manifest, tag says $version"
./scripts/verify-release-versions.sh

echo "== release notes"
notes="docs/releases/$tag.md"
[ -f "$notes" ] || fail "$notes is missing"
grep -q '^## Highlights' "$notes" || fail "$notes has no '## Highlights' section"
grep -q "^## \[$version\]" CHANGELOG.md || fail "CHANGELOG.md has no [$version] section"

echo "== package"
make distcheck

echo "== gate"
make fmt clippy test complexity

echo "preflight ok: push with  git push origin $tag"
