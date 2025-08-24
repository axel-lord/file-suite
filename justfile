crate := "file-suite"
template := "file-suite-template"

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

# Run array expression tests
test-arr-expr:
	cargo test -p array-expr

# Format crates.
fmt:
	cargo fmt --all

# Perform an autoinherit.
autoinherit:
	cargo autoinherit --prefer-simple-dotted

install: autoinherit fmt
	cargo +nightly install --path {{crate}} -Z build-std=std,panic_abort -Z build-std-features="optimize_for_size"

new NAME:
	test ! -e {{NAME}}
	cargo new --lib {{NAME}}
	cp -rT {{template}} {{NAME}}
	fd -tf -e md -e toml -e rs '' {{NAME}} -x sd -F {{template}} {{NAME}}

# Check all features and targets
check:
	cargo clippy --all --all-features --all-targets --workspace
