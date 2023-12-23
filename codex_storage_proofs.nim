
import std/os
import std/strutils
import std/macros

const
  currentDir = currentSourcePath().parentDir()
  libDir* = currentDir/"target"/"release"
  libPath* = libDir/"libcodex_storage_proofs.a"

static:
  let cmd = "cargo build --release"
  warning "\nBuilding codex-storage-proofs: " & cmd
  let (output, exitCode) = gorgeEx cmd
  for ln in output.splitLines():
    warning("cargo> " & ln)
  if exitCode != 0:
    raise (ref Defect)(msg: "Failed to build codex-storage-proofs")


{.passl: "-lcodex_storage_proofs" & " -L" & libDir.}

include codex_proofs_ffi

template unsafeBufferPath*(path: var string): Buffer =
  assert path.len() > 0
  Buffer(data: cast[ptr uint8](path.cstring),
         len: path.len().uint)

template unsafeBufferFromFile*(path: string): Buffer =
  assert path.len() > 0
  let entireFile = readFile(path)

  Buffer(data: cast[ptr uint8](entireFile.cstring),
         len: entireFile.len().uint)

when isMainModule:
  var
    r1cs_path = "src/circuit_tests/artifacts/storer-test.r1cs"
    wasm_path = "src/circuit_tests/artifacts/storer-test_js/storer-test.wasm"

  let
    r1cs_buff = unsafeBufferPath(r1cs_path)
    wasm_buff = unsafeBufferPath(wasm_path)

  let storage_ctx = init_storage_proofs(r1cs_buff, wasm_buff, nil)

  echo "storage_ctx: ", storage_ctx.repr
  assert storage_ctx != nil

  let
    mpack_arg_path = "test/proof_test.mpack"
    proofArgs = unsafeBufferFromFile(mpack_arg_path)
  echo "proofArgs:size: ", proofArgs.len
  # prove_mpack_ext()
