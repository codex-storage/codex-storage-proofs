
task genffi, "update the nim ffi bindings":
  exec "cargo install nbindgen"
  exec "nbindgen --crate codex-storage-proofs --output codex_proofs_ffi.nim"

task test, "run unit tests":
  exec "testament pattern 'test/"
