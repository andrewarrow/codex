#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir"

v8_version="$(sed -nE 's/^v8 = "=([^"]+)"$/\1/p' Cargo.toml | head -n 1)"
if [[ -z "$v8_version" ]]; then
  echo "Could not determine the pinned v8 version from Cargo.toml." >&2
  exit 1
fi

v8_target="aarch64-apple-darwin"
v8_profile="ptrcomp_sandbox_release"
v8_tmp="/private/tmp/codex-v8-${v8_version}"
v8_release_url="https://github.com/openai/codex/releases/download/rusty-v8-v${v8_version}"
v8_archive_name="librusty_v8_${v8_profile}_${v8_target}.a.gz"
v8_binding_name="src_binding_${v8_profile}_${v8_target}.rs"
v8_manifest_name="rusty_v8_${v8_profile}_${v8_target}.sha256"
v8_trusted_manifest="${script_dir}/../third_party/v8/rusty_v8_${v8_version//./_}_release_manifests.sha256"

mkdir -p "$v8_tmp"

download() {
  local url="$1"
  local destination="$2"
  local temporary="${destination}.part"

  echo "Downloading $(basename "$destination")"
  if ! curl --fail --location --retry 3 --retry-delay 2 --silent --show-error \
    "$url" -o "$temporary"; then
    rm -f "$temporary"
    return 1
  fi
  mv "$temporary" "$destination"
}

v8_manifest="${v8_tmp}/${v8_manifest_name}"
v8_archive="${v8_tmp}/${v8_archive_name}"
v8_binding="${v8_tmp}/${v8_binding_name}"

if [[ ! -f "$v8_trusted_manifest" ]]; then
  echo "Missing trusted V8 manifest: $v8_trusted_manifest" >&2
  exit 1
fi

download "${v8_release_url}/${v8_manifest_name}" "$v8_manifest"

expected_manifest_checksum="$(awk -v name="$v8_manifest_name" '$2 == name { print $1; exit }' "$v8_trusted_manifest")"
actual_manifest_checksum="$(shasum -a 256 "$v8_manifest" | awk '{ print $1 }')"
if [[ -z "$expected_manifest_checksum" || "$actual_manifest_checksum" != "$expected_manifest_checksum" ]]; then
  echo "V8 checksum manifest verification failed for $v8_manifest_name." >&2
  exit 1
fi

if [[ ! -f "$v8_archive" ]]; then
  download "${v8_release_url}/${v8_archive_name}" "$v8_archive"
fi
if [[ ! -f "$v8_binding" ]]; then
  download "${v8_release_url}/${v8_binding_name}" "$v8_binding"
fi

(cd "$v8_tmp" && shasum -a 256 -c "$v8_manifest")

export RUSTY_V8_ARCHIVE="$v8_archive"
export RUSTY_V8_SRC_BINDING_PATH="$v8_binding"

cargo build --release \
  -p codex-cli --bin codex \
  -p codex-code-mode-host --bin codex-code-mode-host
