#!/usr/bin/env bash
set -euo pipefail

# Ensure Cargo uses sparse registry protocol and reliable HTTP settings
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export CARGO_HTTP_MULTIPLEXING=false
export CARGO_HTTP_TIMEOUT=120

# Extract the full command string whether invoked as:
# runner.sh "cmd" OR runner.sh -c "cmd" OR runner.sh cmd arg1 arg2
if [[ "${1:-}" == "-c" ]]; then
  shift
fi
cmd="$*"

# Check if this is the locked fetch step
if [[ "$cmd" =~ ^[[:space:]]*prismpm[[:space:]]+fetch[[:space:]]+--locked[[:space:]]*$ ]]; then
  echo "runner: performing deterministic locked preparation"
  mkdir -p .prism/sdk/inputs
  rm -rf test-results playwright-report

  tar_path="/opt/prismpm/share/stdlib-sources.tar"
  if [[ ! -f "$tar_path" && -n "${PRISMPM_SDK_STDLIB_TAR:-}" && -f "${PRISMPM_SDK_STDLIB_TAR}" ]]; then
    tar_path="${PRISMPM_SDK_STDLIB_TAR}"
  elif [[ ! -f "$tar_path" && -n "${PRISMPM_SDK_INVENTORY:-}" && -f "$(dirname "$PRISMPM_SDK_INVENTORY")/stdlib-sources.tar" ]]; then
    tar_path="$(dirname "$PRISMPM_SDK_INVENTORY")/stdlib-sources.tar"
  fi

  if [[ -f "$tar_path" ]]; then
    tar -xf "$tar_path" -C .prism/sdk/inputs
    node scripts/generate-manifest.mjs "$tar_path" .prism/sdk/inputs .prism/sdk/stdlib-manifest.json
  elif [[ -d .prism/sdk/inputs ]]; then
    node scripts/generate-manifest.mjs "" .prism/sdk/inputs .prism/sdk/stdlib-manifest.json
  else
    echo "runner: error: stdlib sources archive not found" >&2
    exit 1
  fi

  if [[ -f Cargo.toml && -f Cargo.lock ]]; then
    cargo fetch --locked
  fi

  if [[ -f package.json && -f package-lock.json ]]; then
    npm ci --ignore-scripts --no-audit --no-fund
  fi

  # Hologram oracle offline harness prefetch
  tar_hologram="/opt/prismpm/share/conformance-root/vendor/hologram-live.tar"
  if [[ ! -f "$tar_hologram" && -f "/home/alex/Desktop/PrismPM/vendor/hologram-live.tar" ]]; then
    tar_hologram="/home/alex/Desktop/PrismPM/vendor/hologram-live.tar"
  fi

  oracle_dir="/opt/prismpm/share/conformance-root/crates/prismpm/src/embedded"
  if [[ ! -d "$oracle_dir" && -d "/home/alex/Desktop/PrismPM/crates/prismpm/src/embedded" ]]; then
    oracle_dir="/home/alex/Desktop/PrismPM/crates/prismpm/src/embedded"
  fi

  if [[ -f "$tar_hologram" && -f "$oracle_dir/hologram-oracle.Cargo.toml" && -f "$oracle_dir/hologram-oracle.Cargo.lock" ]]; then
    echo "runner: prefetching hologram oracle dependencies"
    
    # Check if host or local cache has crates to copy in
    cargo_cache_dir="${CARGO_HOME:-$HOME/.cargo}/registry/cache/index.crates.io-1949cf8c6b5b557f"
    cargo_index_cache="${CARGO_HOME:-$HOME/.cargo}/registry/index/index.crates.io-1949cf8c6b5b557f/.cache"
    mkdir -p "$cargo_cache_dir" "$cargo_index_cache"
    for host_cache in \
      "/home/alex/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f" \
      "$PWD/target/cargo-cache/index.crates.io-1949cf8c6b5b557f"; do
      if [[ -d "$host_cache" && "$host_cache" != "$cargo_cache_dir" ]]; then
        cp -u "$host_cache"/*.crate "$cargo_cache_dir/" 2>/dev/null || true
      fi
    done
    for host_index in \
      "/home/alex/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache" \
      "$PWD/target/cargo-cache/index.crates.io-1949cf8c6b5b557f/.cache"; do
      if [[ -d "$host_index" && "$host_index" != "$cargo_index_cache" ]]; then
        cp -ru "$host_index"/* "$cargo_index_cache/" 2>/dev/null || true
      fi
    done

    oracle_tmp=$(mktemp -d)
    mkdir -p "$oracle_tmp/hologram-live" "$oracle_tmp/harness/src"
    tar -xf "$tar_hologram" -C "$oracle_tmp/hologram-live"
    cp "$oracle_dir/hologram-oracle.Cargo.toml" "$oracle_tmp/harness/Cargo.toml"
    cp "$oracle_dir/hologram-oracle.Cargo.lock" "$oracle_tmp/harness/Cargo.lock"
    touch "$oracle_tmp/harness/src/main.rs"

    cargo fetch --locked --manifest-path "$oracle_tmp/harness/Cargo.toml"
    rm -rf "$oracle_tmp"

    # Save to local target cache to accelerate future local runs if target exists
    if [[ -d "$PWD/target" && "$cargo_cache_dir" != "$PWD/target/cargo-cache/index.crates.io-1949cf8c6b5b557f" ]]; then
      mkdir -p "$PWD/target/cargo-cache/index.crates.io-1949cf8c6b5b557f/.cache"
      cp -u "$cargo_cache_dir"/*.crate "$PWD/target/cargo-cache/index.crates.io-1949cf8c6b5b557f/" 2>/dev/null || true
      cp -ru "$cargo_index_cache"/* "$PWD/target/cargo-cache/index.crates.io-1949cf8c6b5b557f/.cache/" 2>/dev/null || true
    fi
  fi

  echo "runner: locked preparation completed successfully"
  exit 0
fi

exec bash -c "$cmd"
