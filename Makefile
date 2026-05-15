GRADLE_VERSION ?= 9.5.0
RUST_LOG ?= DEBUG

.PHONY: all
all: nitwittery-plugin

gradlew:
	gradle wrapper --gradle-version $(GRADLE_VERSION)

.PHONY: papermc
papermc: gradlew cargo
	./gradlew :papermc:build

.PHONY: nitwittery-plugin
nitwittery-plugin: gradlew cargo
	./gradlew :nitwittery-plugin:build

.PHONY: cargo
cargo:
	cargo build --release

.PHONY: fmt
fmt:
	cargo fmt -- --config group_imports=StdExternalCrate,imports_granularity=Module

.PHONY: fmt-check
fmt-check:
	cargo fmt --check -- --config group_imports=StdExternalCrate,imports_granularity=Module

.PHONY: lint
lint:
	cargo clippy --workspace --all-targets -- -D warnings

.PHONY: test
test:
	cargo test --workspace

.PHONY: run
run: nitwittery-plugin
	RUST_LOG=$(RUST_LOG) ./gradlew :nitwittery-plugin:runServer

.PHONY: clean clean-all
clean:
	cargo clean
	rm -rf ./build/ ./*/bin/
clean-all: clean
	rm -rf ./run/ ./.gradle/ ./gradle/ gradlew gradlew.bat Cargo.lock
