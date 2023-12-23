
task genffi, "update the nim ffi bindings":
    exec "cargo install --force nbindgen"
    exec "nbindgen --crate codex-storage-proofs --output codex_proofs_ffi.nim"
