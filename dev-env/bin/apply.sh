#!/usr/bin/env bash
# Re-apply manifests only (after editing fixtures). Does not touch clusters.
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require kubectl

for c in $CLUSTERS; do
  echo "==> applying manifests for '$c'"
  kubectl --context "kind-$c" apply -k "$DEV_ENV_ROOT/manifests/clusters/$c"
done
