default:
	just --list

# Generate documentation for default feature set.
docs *EXTRA:
	RUSTDOCFLAGS='--cfg=docsrs' cargo +nightly doc -p file-suite {{EXTRA}}

# Generate documentation for all features.
docs-all *EXTRA:
	RUSTDOCFLAGS='--cfg=docsrs' cargo +nightly doc --all-features -p file-suite {{EXTRA}}

# Generate documentation for minimal feature set.
docs-min *EXTRA:
	cargo doc --no-default-features -p file-suite {{EXTRA}}

# Run all tests with all features.
test-all:
	cargo test --all --all-features

# Run tests with all features.
test  *EXTRA:
	cargo test --all-features {{EXTRA}}

# Run tests using miri
test-miri *EXTRA:
	cargo miri test {{EXTRA}}

# Run proc-macro tests
test-proc:
	cargo test -p file-suite-proc-lib -p array-expr -p tokens-rc -p run-derive -p fold-tokens

# Format crates.
fmt:
	cargo fmt --all

# Check all features and targets
check:
	cargo clippy --all --all-features --all-targets --workspace
