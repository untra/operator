# Operator developer tasks.
#
# `make check` mirrors the CI `lint-test` job exactly so a clean local run means
# a clean CI run. `make install-hooks` wires the committed pre-push hook so the
# same gate runs automatically before every push.

.PHONY: check fmt clippy test build run install-hooks

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

# One-time per clone: route git hooks at the committed .githooks/ directory.
install-hooks:
	git config core.hooksPath .githooks
	@echo "pre-push hook installed (runs 'make check')"
