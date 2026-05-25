.PHONY: all build-wasm build-frontend dev clean test

# Build everything: WASM core then React frontend
all: build-wasm build-frontend

# Compile Rust → WASM (bundler target for Vite + vite-plugin-wasm)
build-wasm:
	cd compiler-core && wasm-pack build --target bundler --out-dir ../frontend/src/wasm

# Install npm deps and bundle the frontend
build-frontend:
	cd frontend && npm install && npm run build

# Development: rebuild WASM then start Vite dev server
dev: build-wasm
	cd frontend && npm run dev

# Run all Rust unit + integration tests
test:
	cd compiler-core && cargo test

# Remove generated artifacts
clean:
	cd compiler-core && cargo clean
	cd frontend && rm -rf dist src/wasm
