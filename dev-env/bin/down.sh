#!/usr/bin/env bash
# Delete all local clusters and the isolated kubeconfig.
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require kind
pick_provider

for c in $CLUSTERS; do
  if kind get clusters 2>/dev/null | grep -qx "$c"; then
    echo "==> deleting cluster '$c'"
    kind delete cluster --name "$c"
  fi
done
rm -f "$KUBECONFIG"
