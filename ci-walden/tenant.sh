#!/usr/bin/env bash
# ci-walden tenant-interface v4 — digest-and-verdict drill (positive: plain tree)
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
rm -rf dist
mkdir -p dist/assets
printf 'digest-drill fixed-string v1\n' > dist/server-bin
printf 'body{color:red}\n' > dist/assets/app.css
echo "tenant gate ok"
