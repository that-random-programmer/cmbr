#!/usr/bin/env bash
set -euo pipefail # using nothing special, so its fine to use what the community calls "strict mode"

./compile-web.sh "$@"

cd web

pnpm build