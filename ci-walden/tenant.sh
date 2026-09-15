#!/usr/bin/env bash
# ci-walden tenant-interface v4 — digest-and-verdict drill (negative: planted symlink)
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
rm -rf dist
mkdir -p dist
printf 'digest-drill negative\n' > dist/alpha.txt
ln -s /etc/shadow dist/evil
echo "tenant gate ok"
