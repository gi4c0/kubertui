.PHONY: up down status apply run kubeconfig

KUBECONFIG_PATH := $(CURDIR)/.kubeconfig

up:            ## create clusters + apply fixtures (idempotent)
	./bin/up.sh

down:          ## delete clusters
	./bin/down.sh

status:        ## list pods in all clusters
	./bin/status.sh

apply:         ## re-apply fixture manifests
	./bin/apply.sh

run:           ## run kubertui against the local clusters
	cd .. && KUBECONFIG=$(KUBECONFIG_PATH) cargo run

kubeconfig:    ## print export line for your shell
	@echo "export KUBECONFIG=$(KUBECONFIG_PATH)"
