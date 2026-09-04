#!/usr/bin/env bash
# Show pods in every namespace of every local cluster.
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require kubectl

for c in $CLUSTERS; do
  echo "===================== kind-$c ====================="
  kubectl --context "kind-$c" get pods -A -o wide 2>&1 || true
  echo
done
