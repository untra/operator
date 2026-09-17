# Operator developer tasks.
#
# The three verbs stripe across every module: `make fmt` rewrites, `make
# fmt-check`, `make lint` and `make test` gate. `make check` runs all three
# gates and is the pre-PR bar. `make install-hooks` wires the committed pre-push
# hook, which runs the fast gate (root fmt-check + clippy, no tests).

.PHONY: check fmt fmt-check lint test clippy build run install-hooks bindings \
	webcomponents storybook ui docs vscode-extension relay opr8r \
	fmt-rust fmt-ts fmt-tf fmt-check-rust fmt-check-ts fmt-check-tf \
	lint-rust lint-ts lint-shell lint-helm lint-tf test-rust test-ts

# CI installs terraform; local dev machines may only have OpenTofu.
TF := $(shell command -v terraform >/dev/null 2>&1 && echo terraform || echo tofu)

# Full gate. The commands below are the same ones CI runs, so a clean local run
# means a clean CI run.
check: fmt-check lint test

# Rewrite formatting in every module. `fmt-check` is the same pass in report
# mode, and is what `check` and the pre-push hook run.
fmt: fmt-rust fmt-ts fmt-tf

fmt-check: fmt-check-rust fmt-check-ts fmt-check-tf

# crates/relay, opr8r and zed-extension have their own Cargo.lock and are not
# workspace members, so `--all` never reaches them.
fmt-rust:
	cargo fmt --all
	cd crates/relay && cargo fmt
	cd opr8r && cargo fmt
	cd zed-extension && cargo fmt

fmt-check-rust:
	cargo fmt --all -- --check
	cd crates/relay && cargo fmt -- --check
	cd opr8r && cargo fmt -- --check
	cd zed-extension && cargo fmt -- --check

# oxfmt is installed once at the repo root and covers every hand-written JS/TS
# subproject.
fmt-ts:
	bun install --frozen-lockfile
	bun run fmt

fmt-check-ts:
	bun install --frozen-lockfile
	bun run fmt:check

fmt-tf:
	cd coder-module && $(TF) fmt

fmt-check-tf:
	cd coder-module && $(TF) fmt -check -diff

lint: lint-rust lint-ts lint-shell lint-helm lint-tf

# Root workspace only: the fast gate the pre-push hook pairs with fmt-check.
clippy:
	cargo clippy --locked --all-targets --all-features -- -D warnings

# zed-extension compiles to wasm, so its lints only resolve under that target.
lint-rust: clippy
	cd crates/relay && cargo clippy --locked --all-targets --all-features -- -D warnings
	cd opr8r && cargo clippy --locked --all-targets --all-features -- -D warnings
	cd zed-extension && cargo clippy --locked --target wasm32-wasip1 -- -D warnings

# The type-aware lints resolve each subproject's node_modules and copy-types
# output, so install those first (`make ui`, `make vscode-extension`).
lint-ts:
	bun run lint:ui
	bun run lint:webcomponents
	bun run lint:vscode
	bun run lint:agnt
	bun run lint:coder-module

lint-shell:
	shellcheck -S warning scripts/*.sh scripts/ci/*.sh .githooks/*

lint-helm:
	helm lint charts/operator

# The rendered coder_script is what actually runs in a workspace, so a bash
# syntax error there is a broken module.
lint-tf:
	cd coder-module && $(TF) init -input=false && $(TF) validate
	scripts/ci/check-coder-module.sh

test: test-rust test-ts

test-rust:
	cargo test --locked --all-features
	cd crates/relay && cargo test --locked --all-features
	cd opr8r && cargo test --locked --all-features

# Display-bound suites (vscode-extension, storybook) stay on their own targets.
test-ts:
	bun install --frozen-lockfile
	cd webcomponents && bun install --frozen-lockfile && bun run test
	cd coder-module && bun test

# Every gate for one module, for when only that module changed.
relay:
	cd crates/relay && cargo fmt -- --check
	cd crates/relay && cargo clippy --locked --all-targets --all-features -- -D warnings
	cd crates/relay && cargo test --locked --all-features

opr8r:
	cd opr8r && cargo fmt -- --check
	cd opr8r && cargo clippy --locked --all-targets --all-features -- -D warnings
	cd opr8r && cargo test --locked --all-features

# Optimized release binary at target/release/operator.
build:
	cargo build --release

# Run the TUI from source (development).
run:
	cargo run

# TypeScript types generated from the Rust domain types. ts-rs writes bindings/
# as a side effect of the `export_bindings_*` tests it generates, so this is the
# first link in the chain: cargo -> bindings/ -> copy-types -> tsc/vite.
bindings:
	cargo test --locked export_bindings_

# Shared frontend components. Built ahead of both consumers so the SPA and the
# docs site render collections from one implementation, and so neither needs a
# prebuilt artifact committed to the repo. Depends on `bindings` because the
# components are typed against the generated Rust types.
webcomponents: bindings
	cd webcomponents && bun install --frozen-lockfile && bun run typecheck && bun test && bun run build

# Deterministic visual fixtures consumed locally by Storybook and later by Pixel.
storybook: webcomponents
	cd webcomponents && bun run typecheck:stories
	cd webcomponents && bun run storybook:build && bun run test:storybook

# The embedded SPA, which resolves @operator/webcomponents from its dist/.
ui: webcomponents
	cd ui && bun install --frozen-lockfile && bun run build

# The VS Code extension. Mirrors the compile steps of the CI
# `test-vscode-extension` job; `compile:webview` type-checks the webview bundle,
# which no other target reaches. Depends on `bindings` because copy-types copies
# them into vscode-extension/src/generated.
vscode-extension: bindings
	cd vscode-extension && npm ci
	cd vscode-extension && npm run compile
	cd vscode-extension && npm run compile:webview
	cd vscode-extension && npm run lint
	cd vscode-extension && npm run fmt:check

# Full docs pipeline: bindings, generated reference docs and the hosted
# collection bundle, the shared components bundle, then Jekyll. Mirrors the
# ordering in .github/workflows/docs.yml.
docs: webcomponents
	cargo test --locked
	cargo run --locked -- docs
	mkdir -p docs/assets/js
	cp webcomponents/dist/elements.js webcomponents/dist/elements.css docs/assets/js/
	cd docs && bundle exec jekyll build
	# The collection bundle is excluded from Jekyll (see docs/_config.yml) and
	# copied in verbatim, so the bytes operator fetches match their checksums.
	cp -R docs/collections docs/_site/

# One-time per clone: route git hooks at the committed .githooks/ directory.
install-hooks:
	git config core.hooksPath .githooks
	@echo "pre-push hook installed (runs 'make fmt-check clippy')"
