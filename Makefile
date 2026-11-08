# Single local entry point. `make ci` runs the same commands as .github/workflows/ci.yml,
# in the same order. Keep the two in step.
#
# Prerequisites run one after another under `make ci`. Do not run it with `make -j`,
# because the gates should fail in a fixed order.

SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := ci

AXIOM := projects/axiom
BENCH_LOG := $(AXIOM)/bench-smoke.log

.PHONY: ci fmt fmt-check lint test cov bench wasm deny docs

# Every recipe runs from the repo root and changes into the crate it needs.

fmt:
	cd $(AXIOM) && cargo fmt --all

fmt-check:
	cd $(AXIOM) && cargo fmt --all -- --check

lint:
	cd $(AXIOM) && cargo clippy --all-targets --all-features -- -D warnings

test:
	cd $(AXIOM) && cargo test --all-targets

cov:
	cd $(AXIOM) && cargo llvm-cov --fail-under-lines 90 --fail-under-functions 90 --fail-under-regions 85

bench:
	cd $(AXIOM) && cargo bench --bench belt_throughput -- --quick | tee bench-smoke.log
	bash scripts/ci/check-bench-output.sh $(BENCH_LOG)

wasm:
	cd $(AXIOM) && bash scripts/build-wasm.sh

deny:
	cd $(AXIOM) && cargo deny check

docs:
	npx --yes markdownlint-cli2@0.23.2 "**/*.md"

# Same order as the CI jobs: fmt, clippy, test, coverage, bench-smoke, wasm, supply-chain, docs.
ci: fmt-check lint test cov bench wasm deny docs
