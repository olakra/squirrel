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

# Governance checks. These run the same scripts as the CI jobs commit-lint and changelog-check.
# They need no network. commit-lint and changelog-check compare against BASE_REF (default origin/main).
.PHONY: commit-lint changelog-check release-check scripts-test

BASE_REF ?= origin/main
HEAD_REF ?= HEAD

commit-lint:
	bash scripts/ci/check-commit-messages.sh "$(BASE_REF)" "$(HEAD_REF)"

changelog-check:
	bash scripts/ci/check-changelog.sh "$(BASE_REF)" "$(HEAD_REF)"

# Checks the release tag rules against the Cargo.toml version. It tags nothing.
release-check:
	bash scripts/ci/check-release-tag.sh --local

# Runs every script self-test. Each must exit 0.
scripts-test:
	bash -n scripts/ci/*.sh
	bash scripts/ci/check-commit-messages.sh --self-test
	bash scripts/ci/check-changelog.sh --self-test
	bash scripts/ci/extract-changelog.sh --self-test

# JavaScript checks. They need Bun 1.4.4 on PATH and no network access.
# js-test is not part of `make ci`: ci.yml does not install Bun, so adding it there would break CI.
# The js-test gate runs in .github/workflows/steam-build.yml.
.PHONY: js-test js-check

js-test:
	cd $(AXIOM)/web && bun test
	cd $(AXIOM)/desktop && bun run test

# Parses every hand-written JavaScript file with Bun's transpiler. It bundles and writes nothing.
js-check:
	find $(AXIOM)/web $(AXIOM)/desktop \( -name node_modules -o -name app-web -o -name dist \) -prune -o \( -name '*.js' -o -name '*.mjs' \) -type f -print0 | xargs -0 -n1 bun build --no-bundle > /dev/null
