RUST_LOG ?= DEBUG

.PHONY: all
all: build

.PHONY: build
build: cargo gradle

.PHONY: cargo
cargo:
	cargo build --workspace --release

.PHONY: gradle
gradle:
	./gradlew build

.PHONY: nitwittery-plugin
nitwittery-plugin: cargo
	./gradlew :nitwittery-plugin:build

.PHONY: run
run: nitwittery-plugin
	RUST_LOG=$(RUST_LOG) ./gradlew :nitwittery-plugin:runServer

.PHONY: test
test:
	cargo test --workspace

.PHONY: lint
lint: lint-rust lint-java

.PHONY: lint-rust
lint-rust:
	cargo clippy --workspace --all-targets -- -D warnings

.PHONY: lint-java
lint-java:
	./gradlew spotlessCheck compileJava

.PHONY: fmt
fmt: fmt-rust fmt-other fmt-java

.PHONY: fmt-rust
fmt-rust:
	cargo fmt -- --config group_imports=StdExternalCrate,imports_granularity=Module

.PHONY: fmt-other
fmt-other:
	dprint fmt

.PHONY: fmt-java
fmt-java:
	./gradlew spotlessApply

.PHONY: clean clean-all
clean:
	cargo clean
	rm -rf ./build/ ./*/bin/ .settings/
clean-all: clean
	rm -rf ./run/ ./.gradle/ Cargo.lock
