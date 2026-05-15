GRADLE_VERSION ?= 9.5.0
RUST_LOG ?= DEBUG

.PHONY: all
all: build

# ---- Build / run ----

gradlew:
	gradle wrapper --gradle-version $(GRADLE_VERSION)

.PHONY: build
build: cargo gradle

.PHONY: cargo
cargo:
	cargo build --workspace --release

.PHONY: gradle
gradle: gradlew
	./gradlew build

.PHONY: nitwittery-plugin
nitwittery-plugin: gradlew cargo
	./gradlew :nitwittery-plugin:build

.PHONY: run
run: nitwittery-plugin
	RUST_LOG=$(RUST_LOG) ./gradlew :nitwittery-plugin:runServer

# ---- Test ----

.PHONY: test
test:
	cargo test --workspace

# ---- Lint ----

.PHONY: lint
lint: lint-rust lint-java

.PHONY: lint-rust
lint-rust:
	cargo clippy --workspace --all-targets -- -D warnings

.PHONY: lint-java
lint-java: gradlew
	./gradlew spotlessCheck compileJava

# ---- Format ----

.PHONY: fmt
fmt: fmt-rust fmt-other fmt-java

.PHONY: fmt-rust
fmt-rust:
	cargo fmt -- --config group_imports=StdExternalCrate,imports_granularity=Module

.PHONY: fmt-other
fmt-other:
	dprint fmt

.PHONY: fmt-java
fmt-java: gradlew
	./gradlew spotlessApply

# ---- Clean ----

.PHONY: clean clean-all
clean:
	cargo clean
	rm -rf ./build/ ./*/bin/ .settings/
clean-all: clean
	rm -rf ./run/ ./.gradle/ ./gradle/ gradlew gradlew.bat Cargo.lock
