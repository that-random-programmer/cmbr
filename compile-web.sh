#!/usr/bin/env bash
set -euo pipefail # using nothing special, so its fine to use what the community calls "strict mode"

wasm-pack build "$@" --target web -- --features wasm
mkdir -p web/src/lib/pkg # TODO make this better ig
rm -r web/src/lib/pkg
mv pkg web/src/lib/pkg 
