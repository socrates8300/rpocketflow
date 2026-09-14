#!/usr/bin/env bash
# ci-walden tenant-interface v4 — JUDGE PROBE P2 (negative: planted hardlink)
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
rm -rf dist
mkdir -p dist
printf 'judge-probe p2 fixed-string\n' > dist/alpha.txt
ln dist/alpha.txt dist/hardlink
echo "tenant gate ok"
