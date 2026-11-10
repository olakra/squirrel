# Single local entry point. `make ci` runs the same commands as .github/workflows/ci.yml,
# in the same order. Keep the two in step. `make help` lists every target.
#
# Prerequisites run one after another under `make ci`. Do not run it with `make -j`,
# because the gates should fail in a fixed order.
#
# Branching and releases: docs/engineering/branching-strategy.md and
# docs/engineering/release-process.md. Targets marked "(owner)" create tags, push shared refs,
# or change GitHub settings. Agents never run them (CLAUDE.md).

# Recipes need bash. On Windows, "bash" on PATH is often C:\Windows\System32\bash.exe, the WSL
# launcher, which fails without a WSL distro. So use Git for Windows' bash there. The 8.3 short
# path avoids the space in "Program Files", which make cannot handle in SHELL. Override it with
# `make GIT_BASH=D:/Tools/Git/bin/bash.exe ...` (forward slashes, no spaces) if Git is elsewhere.
ifeq ($(OS),Windows_NT)
  GIT_BASH ?= C:/PROGRA~1/Git/bin/bash.exe
  ifeq ($(wildcard $(GIT_BASH)),)
    $(error Git Bash not found at $(GIT_BASH). Install Git for Windows, or pass GIT_BASH=<path to bash.exe>)
  endif
  SHELL := $(GIT_BASH)
else
  SHELL := /bin/bash
endif
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := ci

AXIOM := projects/axiom
TICTACTOE := projects/tictactoe
BENCH_LOG := $(AXIOM)/bench-smoke.log

# Release and branch variables. PROJECT is one of the ids in scripts/release/projects.sh.
PROJECT ?=
VERSION ?=
BASE ?= origin/main
FROM ?=
NAME ?=
TYPE ?= feature
HEAD_REF ?= HEAD

# Stops with a message when a required variable is empty. Usage: $(call require,PROJECT)
require = $(if $(strip $($(1))),,$(error $(1) is required, for example: make $@ $(1)=<value>))

##@ Quality gates (same as CI)

.PHONY: ci fmt fmt-check lint test cov bench wasm deny docs tictactoe-check tictactoe-web

fmt: ## Format the axiom and tictactoe crates
	cd $(AXIOM) && cargo fmt --all
	cd $(TICTACTOE) && cargo fmt --all

fmt-check: ## Fail on unformatted axiom code
	cd $(AXIOM) && cargo fmt --all -- --check

lint: ## clippy on axiom, warnings denied
	cd $(AXIOM) && cargo clippy --all-targets --all-features -- -D warnings

test: ## axiom tests
	cd $(AXIOM) && cargo test --all-targets

cov: ## axiom coverage gate (needs cargo-llvm-cov)
	cd $(AXIOM) && cargo llvm-cov --fail-under-lines 90 --fail-under-functions 90 --fail-under-regions 85

bench: ## axiom benchmark smoke run
	cd $(AXIOM) && cargo bench --bench belt_throughput -- --quick | tee bench-smoke.log
	bash scripts/ci/check-bench-output.sh $(BENCH_LOG)

wasm: ## Build axiom.wasm
	cd $(AXIOM) && bash scripts/build-wasm.sh

deny: ## Supply-chain gate (needs cargo-deny)
	cd $(AXIOM) && cargo deny check

docs: ## markdownlint every Markdown file
	npx --yes markdownlint-cli2@0.23.2 "**/*.md"

tictactoe-check: ## tictactoe fmt check, clippy and tests (CI job "tictactoe", minus the bundle)
	cd $(TICTACTOE) && cargo fmt --all -- --check
	cd $(TICTACTOE) && cargo clippy --all-targets --all-features -- -D warnings
	cd $(TICTACTOE) && cargo test --workspace

tictactoe-web: ## Build the tictactoe browser bundle (needs wasm-bindgen-cli 0.2.100)
	bash $(TICTACTOE)/web/build.sh

# Same order as the CI jobs: fmt, clippy, test, coverage, bench-smoke, wasm, tictactoe,
# supply-chain, docs, scripts-test.
ci: fmt-check lint test cov bench wasm tictactoe-check deny docs scripts-test ## Run every CI gate in order

##@ Governance checks (no network)

.PHONY: commit-lint changelog-check branch-check scripts-test

commit-lint: ## Check commit subjects since BASE (default origin/main)
	bash scripts/ci/check-commit-messages.sh "$(BASE)" "$(HEAD_REF)"

changelog-check: ## Check every changed project has a changelog entry since BASE
	bash scripts/ci/check-changelog.sh "$(BASE)" "$(HEAD_REF)"

branch-check: ## Check the current branch name against the branching rules
	bash scripts/ci/check-branch-name.sh

scripts-test: ## Run every governance and release script self-test
	bash -n scripts/ci/*.sh scripts/release/*.sh
	bash scripts/ci/check-commit-messages.sh --self-test
	bash scripts/ci/check-changelog.sh --self-test
	bash scripts/ci/extract-changelog.sh --self-test
	bash scripts/ci/check-release-tag.sh --self-test
	bash scripts/ci/check-branch-name.sh --self-test

##@ Branches

.PHONY: branch versions

branch: ## Start a work branch from origin/main: make branch TYPE=feature NAME=belt-sleep
	$(call require,NAME)
	bash scripts/ci/check-branch-name.sh "$(TYPE)/$(NAME)" >/dev/null
	git fetch --quiet origin
	git switch --no-track -c "$(TYPE)/$(NAME)" origin/main
	@echo "On $(TYPE)/$(NAME). Push with: git push -u origin $(TYPE)/$(NAME)"

versions: ## Print each project's version and latest release tag
	@source scripts/release/projects.sh; \
	for p in "$${RELEASE_PROJECTS[@]}"; do \
	  printf '%-10s %-12s latest tag: %s\n' "$$p" "$$(release_read_version $$p)" "$$(release_previous_tag $$p)"; \
	done

##@ Releases

.PHONY: release-prepare release-pr release-check release-tag release-push hotfix-start release-artifacts

release-prepare: ## Branch, cut the changelog, bump versions, commit: PROJECT= VERSION= [BASE=]
	$(call require,PROJECT)
	$(call require,VERSION)
	bash scripts/release/prepare.sh "$(PROJECT)" "$(VERSION)" "$(BASE)"

release-pr: ## Push release/<project>-v<version> and open its PR: PROJECT= VERSION= [BASE=]
	$(call require,PROJECT)
	$(call require,VERSION)
	git push -u origin "release/$(PROJECT)-v$(VERSION)"
	gh pr create --head "release/$(PROJECT)-v$(VERSION)" --base "$(patsubst origin/%,%,$(BASE))" \
	  --title "chore($(PROJECT)): release v$(VERSION)" --fill-first

release-check: ## Check the version files and changelog of PROJECT agree. Tags nothing
	$(call require,PROJECT)
	bash scripts/ci/check-release-tag.sh --local "$(PROJECT)"

release-tag: ## (owner) Create the signed tag on the merged release commit: PROJECT=
	$(call require,PROJECT)
	bash scripts/release/tag.sh "$(PROJECT)"

release-push: ## (owner) Push the release tag of PROJECT; release.yml publishes it
	$(call require,PROJECT)
	@source scripts/release/projects.sh; \
	tag="$$(release_tag_name $(PROJECT) "$$(release_read_version $(PROJECT))")"; \
	git rev-parse -q --verify "refs/tags/$$tag" >/dev/null \
	  || { echo "error: no local tag $$tag. Run: make release-tag PROJECT=$(PROJECT)" >&2; exit 1; }; \
	echo "pushing $$tag"; \
	git push origin "refs/tags/$$tag"

hotfix-start: ## (owner) Support branch from a release, plus a hotfix branch: PROJECT= FROM= NAME=
	$(call require,PROJECT)
	$(call require,FROM)
	$(call require,NAME)
	bash scripts/release/hotfix.sh "$(PROJECT)" "$(FROM)" "$(NAME)"

release-artifacts: ## Build dist/ and SHA256SUMS for PROJECT at VERSION (used by release.yml)
	$(call require,PROJECT)
	$(call require,VERSION)
	bash scripts/release/build-artifacts.sh "$(PROJECT)" "$(VERSION)" dist

##@ GitHub settings (owner)

.PHONY: github-protect-dry github-protect

github-protect-dry: ## Print the rulesets, merge settings and environment that would be applied
	bash scripts/ci/apply-branch-protection.sh --dry-run

github-protect: ## (owner) Apply rulesets, squash-only merge settings and the production environment
	bash scripts/ci/apply-branch-protection.sh --apply

##@ JavaScript (needs Bun 1.4.4, no network)

# js-test is not part of `make ci`: ci.yml does not install Bun, so adding it there would break CI.
# The js-test gate runs in .github/workflows/steam-build.yml.
.PHONY: js-test js-check

js-test: ## Bun tests for axiom web/ and desktop/
	cd $(AXIOM)/web && bun test
	cd $(AXIOM)/desktop && bun run test

js-check: ## Parse every hand-written .js and .mjs file
	find $(AXIOM)/web $(AXIOM)/desktop \( -name node_modules -o -name app-web -o -name dist \) -prune -o \( -name '*.js' -o -name '*.mjs' \) -type f -print0 | xargs -0 -n1 bun build --no-bundle > /dev/null

##@ Help

.PHONY: help

help: ## List the targets
	@awk 'BEGIN { FS = ":.*## " } \
	  /^##@/ { printf "\n%s\n", substr($$0, 5); next } \
	  /^[a-zA-Z0-9_-]+:.*## / { printf "  %-20s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
