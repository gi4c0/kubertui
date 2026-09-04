#!/usr/bin/env bash
# Create kind clusters (idempotent) and apply the fixture manifests for each.
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require kind kubectl
pick_provider

for c in $CLUSTERS; do
  if kind get clusters 2>/dev/null | grep -qx "$c"; then
    echo "==> cluster '$c' already exists, skipping create"
  else
    echo "==> creating cluster '$c'"
    kind create cluster --name "$c" --config "$DEV_ENV_ROOT/kind/cluster.yaml" --wait 180s
  fi

  echo "==> applying manifests for '$c'"
  kubectl --context "kind-$c" apply -k "$DEV_ENV_ROOT/manifests/clusters/$c"
done

first="${CLUSTERS%% *}"
kubectl config use-context "kind-$first" >/dev/null

cat <<MSG

Done. Clusters: $(kind get clusters | tr '\n' ' ')
Kubeconfig:     $KUBECONFIG

Run the TUI against them:
  KUBECONFIG=$KUBECONFIG cargo run
or:
  make -C dev-env run

Pods need a minute or two to pull images. Watch with: make -C dev-env status
MSG
