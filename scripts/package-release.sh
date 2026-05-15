#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
fi
version="${version#v}"

target="${TARGET:-$(rustc -vV | sed -n 's/^host: //p')}"
binary="target/${target}/release/handoff"
archive="handoff-v${version}-${target}.tar.gz"

cargo build --release --target "${target}"

mkdir -p dist
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

cp "${binary}" "${tmpdir}/handoff"
tar -C "${tmpdir}" -czf "dist/${archive}" handoff

if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "dist/${archive}" > "dist/${archive}.sha256"
else
  sha256sum "dist/${archive}" > "dist/${archive}.sha256"
fi

cat "dist/${archive}.sha256"
