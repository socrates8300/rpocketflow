#!/usr/bin/env bash
# ci-walden tenant-interface v4 — QA artifact drill (negative: planted symlink)
# contract: bash at repo root; exit status is the verdict (0 = green)
set -e
mkdir -p dist
ln -sf /etc/hostname dist/evil
printf 'artifact-drill qa-artifacts-lane run3 negative\n' > dist/artifact.txt
echo "tenant gate ok"
