#!/usr/bin/env bash
# ci-walden tenant-interface v4 — JUDGE PROBE (novel tree): nested files + empty file
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
rm -rf dist
mkdir -p dist/nested
printf 'judge-probe alpha fixed-string\n' > dist/alpha.txt
printf 'judge-probe nested fixed-string\n' > dist/nested/deep.txt
: > dist/empty.dat
head -c 1024 /dev/zero | tr '\0' 'J' > dist/nested/pad.bin
echo "tenant gate ok"
