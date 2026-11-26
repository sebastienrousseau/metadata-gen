#!/bin/sh
# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Checks a running deploy sample (#63) answers the documented contract:
#   POST YAML front matter  -> 200 {"author.name":"Ada","title":"Hi"}
#   POST no front matter    -> 422 {"error":...}
#   GET                     -> 405
# Usage: examples/deploy/smoke.sh http://127.0.0.1:3000/
# Waits up to 60 s for the sample to answer GET with 405. Any answer
# is not enough: `wash dev` opens its port before the component is
# loaded and answers 404 until it is.
set -eu
url="${1:?usage: smoke.sh <url>}"
i=0
until [ "$(curl -s -o /dev/null -w '%{http_code}' "$url" 2>/dev/null)" = 405 ]; do
  i=$((i + 1))
  [ "$i" -lt 60 ] || { echo "no sample answering at $url after 60 s" >&2; exit 1; }
  sleep 1
done
fail=0
check() {
  expect_code="$1" expect_body="$2"; shift 2
  out="$(curl -s -w '\n%{http_code}' "$@" "$url")"
  code="$(printf '%s\n' "$out" | tail -n 1)"
  body="$(printf '%s\n' "$out" | sed '$d')"
  # expect_body is a glob on purpose ('{"error":*}'), so it is unquoted.
  # shellcheck disable=SC2254
  case "$body" in
    $expect_body) [ "$code" = "$expect_code" ] && { echo "ok   $code $body"; return; } ;;
  esac
  echo "FAIL want $expect_code $expect_body, got $code $(printf '%s' "$body" | head -c 200)" >&2
  fail=1
}
doc='---
title: Hi
author:
  name: Ada
---
Body'
check 200 '{"author.name":"Ada","title":"Hi"}' -X POST --data-binary "$doc"
check 422 '{"error":*}' -X POST --data-binary 'no front matter'
check 405 'POST a Markdown document' -X GET
exit "$fail"
