#!/usr/bin/env bash
# ci-walden tenant-interface v4 — remediation drill P1 (positive: tricky-name tree)
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
rm -rf dist
mkdir -p dist/assets/deep
printf 'remediation p1: nested\n' > dist/assets/deep/nested.txt
printf 'remediation p1: name with space\n' > 'dist/file with space.txt'
printf 'remediation p1: name with newline\n' > $'dist/file\nname.txt'
: > dist/empty.bin
echo "tenant gate ok"
