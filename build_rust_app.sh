#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust/Cargo was not found. Install it from https://rustup.rs/ first." >&2
  exit 1
fi

cargo build --workspace --release
mkdir -p dist
cp target/release/tf2-frag-helper dist/TF2_Frag_Demo_Helper
cp target/release/export_all dist/export_all
mkdir -p dist/recording_resources_archive
cp recording_resources_archive/resources.part* dist/recording_resources_archive/
echo "Built the GUI, parser helper, and recording resources in dist/"
