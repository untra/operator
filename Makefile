# Operator developer tasks.
#
# `make check` mirrors the CI `lint-test` job exactly so a clean local run means
# a clean CI run. `make install-hooks` wires the committed pre-push hook, which
# runs the fast lint gate (fmt + clippy, no tests) before every push.

.PHONY: check fmt clippy test build run install-hooks bindings webcomponents ui docs

# Full CI-parity gate. Keep these commands byte-identical to
# .github/workflows/build.yaml so local and CI never disagree.
check: fmt clippy test

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --locked --all-targets --all-features -- -D warnings

test:
	cargo test --locked

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

# The embedded SPA, which resolves @operator/webcomponents from its dist/.
ui: webcomponents
	cd ui && bun install --frozen-lockfile && bun run build

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
	@echo "pre-push hook installed (runs 'make fmt clippy')"
