v8_tmp=/private/tmp/codex-v8-150.4.0

  export RUSTY_V8_ARCHIVE="$v8_tmp/librusty_v8_ptrcomp_sandbox_release_aarch64-apple-darwin.a.gz"
  export RUSTY_V8_SRC_BINDING_PATH="$v8_tmp/src_binding_ptrcomp_sandbox_release_aarch64-apple-darwin.rs"

  cargo build --release \
    -p codex-cli --bin codex \
    -p codex-code-mode-host --bin codex-code-mode-host
