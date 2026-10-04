#!/usr/bin/env bash
set -euo pipefail

# Ensure Cargo uses sparse registry protocol and reliable HTTP settings
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export CARGO_HTTP_MULTIPLEXING=false
export CARGO_HTTP_TIMEOUT=120
export npm_config_cache="${TMPDIR:-/tmp}/npm-cache"

# Extract the full command string whether invoked as:
# runner.sh "cmd" OR runner.sh -c "cmd" OR runner.sh cmd arg1 arg2
if [[ "${1:-}" == "-c" ]]; then
  shift
fi
cmd="$*"

# Check if this is the locked fetch step
if [[ "$cmd" =~ ^[[:space:]]*prismpm[[:space:]]+fetch[[:space:]]+--locked[[:space:]]*$ ]]; then
  echo "runner: performing deterministic locked preparation"
  chmod -R u+w .prism/sdk/inputs 2>/dev/null || true
  rm -rf .prism/sdk/inputs test-results playwright-report 2>/dev/null || true
  mkdir -p .prism/sdk/inputs

  tar_path="/opt/prismpm/share/stdlib-sources.tar"
  if [[ ! -f "$tar_path" && -n "${PRISMPM_SDK_STDLIB_TAR:-}" && -f "${PRISMPM_SDK_STDLIB_TAR}" ]]; then
    tar_path="${PRISMPM_SDK_STDLIB_TAR}"
  elif [[ ! -f "$tar_path" && -n "${PRISMPM_SDK_INVENTORY:-}" && -f "$(dirname "$PRISMPM_SDK_INVENTORY")/stdlib-sources.tar" ]]; then
    tar_path="$(dirname "$PRISMPM_SDK_INVENTORY")/stdlib-sources.tar"
  fi

  if [[ -f "$tar_path" ]]; then
    chmod -R u+w .prism/sdk/inputs 2>/dev/null || true
    tar --overwrite -xf "$tar_path" -C .prism/sdk/inputs
    node scripts/generate-manifest.mjs "$tar_path" .prism/sdk/inputs .prism/sdk/stdlib-manifest.json
  else
    echo "runner: error: stdlib sources archive not found at ${tar_path}" >&2
    exit 1
  fi

  if [[ -f Cargo.toml && -f Cargo.lock ]]; then
    cargo fetch --locked
  fi

  if [[ -f package.json && -f package-lock.json ]]; then
    npm ci --cache "${TMPDIR:-/tmp}/npm-cache" --ignore-scripts --no-audit --no-fund
  fi

  # Hologram oracle offline harness prefetch
  tar_hologram="${PRISMPM_CONFORMANCE_ROOT:-/opt/prismpm/share/conformance-root}/vendor/hologram-live.tar"
  oracle_dir="${PRISMPM_CONFORMANCE_ROOT:-/opt/prismpm/share/conformance-root}/crates/prismpm/src/embedded"

  if [[ -f "$tar_hologram" && -f "$oracle_dir/hologram-oracle.Cargo.toml" && -f "$oracle_dir/hologram-oracle.Cargo.lock" ]]; then
    echo "runner: prefetching hologram oracle dependencies"
    
    # Check if local cache has crates to copy in
    cargo_cache_dir="${CARGO_HOME:-$HOME/.cargo}/registry/cache/index.crates.io-1949cf8c6b5b557f"
    cargo_index_cache="${CARGO_HOME:-$HOME/.cargo}/registry/index/index.crates.io-1949cf8c6b5b557f/.cache"
    mkdir -p "$cargo_cache_dir" "$cargo_index_cache"
    for host_cache in \
      "$PWD/target/cargo-cache/index.crates.io-1949cf8c6b5b557f"; do
      if [[ -d "$host_cache" && "$host_cache" != "$cargo_cache_dir" ]]; then
        cp -u "$host_cache"/*.crate "$cargo_cache_dir/" 2>/dev/null || true
      fi
    done
    for host_index in \
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

if [[ "$cmd" =~ ^[[:space:]]*prismpm[[:space:]]+(template|lock)[[:space:]]+check ]] && ! test -f /.dockerenv; then
  exec docker run --rm -u 1000:1000 -v "$PWD:/work" -w /work ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21 bash -c "$cmd"
fi

exec bash -c "$cmd"
