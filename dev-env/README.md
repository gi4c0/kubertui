# Local dev environment

Fake "multi-cluster" setup for developing kubertui on a machine without access to the real clusters.
Three [kind](https://kind.sigs.k8s.io/) clusters, each with a different set of namespaces, log producers,
broken pods and databases. Everything the TUI does (list clusters, switch context, list namespaces/pods,
read logs, port-forward, delete pods) works against it.

The kubeconfig is isolated in `dev-env/.kubeconfig` (git-ignored), so nothing touches `~/.kube/config`.

## 1. Install

You need: a container runtime, `kind`, `kubectl`, and the Rust toolchain for the app itself.

### macOS

```sh
brew install kind kubectl
```

Container runtime, pick one:

| Option | Install | Notes |
|---|---|---|
| OrbStack (recommended) | `brew install orbstack` | Fastest, lightest, docker-compatible. kind works out of the box. |
| Docker Desktop | https://www.docker.com/products/docker-desktop | Fine. Give it 4 CPU / 8 GB in Settings → Resources. |
| Podman | `brew install podman` then see below | Works, kind uses its experimental podman provider. |

Podman setup (skip if using OrbStack/Docker):

```sh
podman machine init --cpus 4 --memory 8192 --disk-size 60 --rootful
podman machine start
```

`--rootful` matters: kind needs cgroup delegation that rootless podman machine does not give reliably.
If you already have a machine: `podman machine stop && podman machine set --rootful --cpus 4 --memory 8192 && podman machine start`.
The scripts detect podman automatically when docker is absent and export `KIND_EXPERIMENTAL_PROVIDER=podman`.

### Linux

```sh
# kind
curl -Lo ./kind https://kind.sigs.k8s.io/dl/latest/kind-linux-$(dpkg --print-architecture 2>/dev/null || uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')
chmod +x ./kind && sudo mv ./kind /usr/local/bin/kind
# kubectl
curl -LO "https://dl.k8s.io/release/$(curl -Ls https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl && sudo mv kubectl /usr/local/bin/
# docker: https://docs.docker.com/engine/install/  (or `sudo apt install podman`)
```

### Sizing

Three clusters plus four databases need roughly 6 GB RAM in the VM/daemon. If the machine is small,
run one cluster: `CLUSTERS=dev make up`. `dev` contains every fixture.

## 2. Bring it up

```sh
cd dev-env
make up          # creates kind-dev, kind-stage, kind-prod and applies fixtures (idempotent)
make status      # pods across all clusters; wait until images are pulled (~1-3 min first time)
make run         # cargo run with KUBECONFIG pointed at the local clusters
```

Or from your own shell:

```sh
eval "$(make -s -C dev-env kubeconfig)"   # export KUBECONFIG=.../dev-env/.kubeconfig
cargo run
kubectl get pods -A                       # plain kubectl also works against the same clusters
```

Other targets:

| Target | Does |
|---|---|
| `make apply` | Re-apply manifests after editing fixtures. No cluster recreate. |
| `make down` | Delete all clusters and the kubeconfig. |
| `CLUSTERS="dev stage" make up` | Only some clusters. Same var works for all targets. |

Cluster names as the TUI sees them: `kind-dev`, `kind-stage`, `kind-prod` (kind prefixes the name,
and context name == cluster name, which is what the app assumes).

## 3. What is in the clusters

| Namespace | dev | stage | prod | Contents |
|---|:-:|:-:|:-:|---|
| `shop` | ✓ | ✓ | ✓ | `orders-api` ×2 (JSON logs), `inventory-service` (text logs, stack traces, 600-char lines) |
| `platform` | ✓ | ✓ | ✓ | container-shape fixtures, `firehose`, `color-logger`, `silent` |
| `payments` | ✓ | ✓ | ✓ | `payments-api` ×3 (JSON), `payments-db` (postgres) |
| `databases` | ✓ | ✓ | | postgres, mysql, redis, mongo |
| `monitoring` | ✓ | | ✓ | `metrics-agent` DaemonSet |
| `chaos` | ✓ | | | every broken pod state |

### Fixture → what it exercises in the app

| Pod (namespace) | Exercises |
|---|---|
| `orders-api` (shop) | JSON log lines, 2 replicas of one app, `containerPort` 8080 |
| `inventory-service` (shop) | Multi-line stack traces, WARN/ERROR levels, very long lines |
| `firehose` (platform) | ~50 lines/s, scroll and memory behaviour |
| `color-logger` (platform) | ANSI escape codes in log output |
| `silent` (platform) | Empty log output |
| `gateway` (platform) | 2 containers with `default-container` annotation, sidecar has port 15000 |
| `gateway-nodefault` (platform) | 2 containers, no annotation → `kubectl logs <pod>` **errors** without `-c`. Current `load_logs` will fail here. |
| `many-containers` (platform) | 6 containers, one crash-looping → status cell shows `5/6` |
| `no-ports` (platform) | Container without `ports` → port 0 in the port-forward popup |
| `multi-port` (platform) | 3 named ports, busybox httpd on 8080 → forward and `curl localhost:<port>` |
| `crashloop` (chaos) | Terminated(Error) ↔ Waiting(CrashLoopBackOff) 💔/💤, restart count |
| `image-pull-fail` (chaos) | Waiting(ErrImagePull / ImagePullBackOff) 💤 |
| `pending-unschedulable` (chaos) | Pending, `containerStatuses` null, no `reason` → empty status cell |
| `init-stuck` (chaos) | Waiting(PodInitializing) 💤 forever |
| `flaky` (chaos) | Running but exits 0 every minute → restart count climbs while green |
| `oom` (chaos) | Terminated(OOMKilled) 💔 |
| `completed-job` (chaos) | Job pod, Terminated(Completed) 💔 |
| `failed-job` (chaos) | Job pod, Terminated(Error) 💔 |
| `hourly-report-*` (chaos) | CronJob every 2 min, keeps last 3 Completed pods → list churn |
| `metrics-agent-*` (monitoring) | DaemonSet pod naming |

Not reproducible with kind: `Evicted` (`status.reason`). You would need real node memory/disk pressure.

### Databases and port-forward

Forward from the TUI (or `kubectl -n databases port-forward deploy/postgres 5432:5432`) then connect:

| DB | Namespace / pod | Port | Connect |
|---|---|---|---|
| PostgreSQL 16 | `databases/postgres` | 5432 | `psql "postgres://app:secret@localhost:5432/shop"` |
| PostgreSQL 16 | `payments/payments-db` | 5432 | `psql "postgres://payments:secret@localhost:5432/payments"` |
| MySQL 8.4 | `databases/mysql` | 3306 | `mysql -h 127.0.0.1 -P 3306 -u app -psecret shop` (root: `secret`) |
| Redis 7 | `databases/redis` | 6379 | `redis-cli -p 6379 -a secret ping` |
| MongoDB 7 | `databases/mongo` | 27017 | `mongosh "mongodb://app:secret@localhost:27017"` |
| busybox httpd | `platform/multi-port` | 8080 | `curl localhost:8080` |

Data is `emptyDir`, gone when the pod restarts. That is intentional.

## 4. Troubleshooting

| Symptom | Fix |
|---|---|
| `kind create cluster` hangs at "Starting control-plane" on podman | Machine is rootless. `podman machine set --rootful` (stop it first). |
| `error: need a running docker or podman` | `podman machine start` or start Docker/OrbStack. |
| `mongo` pod CrashLoopBackOff on an Intel Mac | MongoDB 5+ requires AVX. Old Intel CPUs lack it. Delete the mongo deployment or pin `mongo:4.4`. |
| `mysql` pod slow to become Ready | Normal, ~30-60 s on first start. |
| kubectl warns about version skew | Harmless. To silence, `kind create cluster --image kindest/node:<version>` closer to your kubectl. |
| Pods stuck `ContainerCreating` for minutes | Image pulls inside the kind node. Check `kubectl -n <ns> describe pod <pod>`. |
| Want a clean slate | `make down && make up`. |

## 5. Layout

```
dev-env/
  Makefile, bin/            up / down / status / apply scripts, KUBECONFIG + provider detection
  kind/cluster.yaml         node layout (single control-plane)
  manifests/components/     reusable fixture bundles, each with a kustomization.yaml
    apps/ chaos/ databases/ payments/ monitoring/
  manifests/clusters/       one kustomization per cluster picking components
    dev/ stage/ prod/
```

Add a fixture: drop YAML into a component, list it in that component's `kustomization.yaml`,
`make apply`. Add a cluster: new dir under `manifests/clusters/`, add its name to `CLUSTERS`.
