
task genffi, "update the nim ffi bindings":
  exec "cargo install nbindgen"
  exec "nbindgen --crate codex-storage-proofs --output codex_proofs_ffi.nim"

task compileCircuits, "compile test circuits":
  exec "circom src/circuit_tests/poseidon-digest-test.circom --r1cs --wasm -o src/circuit_tests/artifacts"
  exec "circom src/circuit_tests/poseidon-hash-test.circom --r1cs --wasm -o src/circuit_tests/artifacts"
  exec "circom src/circuit_tests/storer-test.circom --r1cs --wasm -o src/circuit_tests/artifacts"

task tests, "run unit tests":
  exec "nim c -r tests/tffi.nim"
