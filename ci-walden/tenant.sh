#!/usr/bin/env bash
# ci-walden tenant-interface v4 — judge probe JP2 (negative: planted SYMLINK, type 50)
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
rm -rf dist
mkdir -p dist
printf 'judge p2 fixed-string\n' > dist/alpha.txt
ln -s /etc/passwd dist/evil
echo "tenant gate ok"
