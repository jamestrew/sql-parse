.PHONY: test fmt lint

all: test fmt lint

test:
	@echo "Running tests..."
	@cargo test --verbose

fmt:
	@echo "Checking formatting..."
	@cargo fmt --all

lint:
	@echo "Linting with clippy..."
	@cargo clippy --all-targets -- -D warnings

build:
	@cargo build --target x86_64-unknown-linux-musl --release
	tar czf sql-parse.tar.gz ./target/x86_64-unknown-linux-musl/release/sql-parse
