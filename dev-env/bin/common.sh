# shellcheck shell=bash
# Sourced by the other scripts. Sets KUBECONFIG and picks a container runtime for kind.
set -euo pipefail

DEV_ENV_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export KUBECONFIG="$DEV_ENV_ROOT/.kubeconfig"

# Space separated. Override: CLUSTERS="dev" ./bin/up.sh
CLUSTERS="${CLUSTERS:-dev stage prod}"

pick_provider() {
  if [ -n "${KIND_EXPERIMENTAL_PROVIDER:-}" ]; then
    return
  fi
  if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    return # kind default
  fi
  if command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; then
    export KIND_EXPERIMENTAL_PROVIDER=podman
    return
  fi
  echo "error: need a running docker or podman (podman machine start)" >&2
  exit 1
}

require() {
  for bin in "$@"; do
    command -v "$bin" >/dev/null 2>&1 || { echo "error: '$bin' not installed" >&2; exit 1; }
  done
}
