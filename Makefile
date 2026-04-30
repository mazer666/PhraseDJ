# PhraseDJ — top-level convenience targets.
# These mirror what the CI runs so you can check locally before pushing.

.PHONY: ci fmt lint test test-cpp file-length clean

# Run the full quality bar (same gates as CI).
ci: fmt lint test file-length

# Format all code.
fmt:
	cargo fmt --all
	cd apps/desktop && pnpm exec prettier --write "src/**/*.{ts,tsx,css}"

# Lint all code.
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cd apps/desktop && pnpm exec tsc --noEmit

# Run Rust + frontend tests.
test:
	cargo test --workspace --all-features
	cd apps/desktop && pnpm test

# Build and run C++ tests (requires cmake).
test-cpp:
	cmake -B native/audio/build native/audio -DCMAKE_BUILD_TYPE=Release \
	      -DCMAKE_CXX_COMPILER=clang++
	cmake --build native/audio/build --parallel
	cd native/audio/build && ctest --output-on-failure

# File-length hard-limit check.
file-length:
	bash tools/check_file_size.sh

clean:
	cargo clean
	rm -rf apps/desktop/dist apps/desktop/node_modules
	rm -rf native/audio/build
