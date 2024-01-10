
import std/os
import unittest2
import codex_storage_proofs

suite "storage proofs ffi":
  test "basic ffi circuit":
    var
      r1csPath = "src/circuit_tests/artifacts/storer-test.r1cs".absolutePath()
      wasmPath = "src/circuit_tests/artifacts/storer-test_js/storer-test.wasm".absolutePath()

    assert r1csPath.fileExists()
    assert wasmPath.fileExists()
    let
      r1cs_buff = unsafeBufferPath(r1csPath)
      wasm_buff = unsafeBufferPath(wasmPath)

    let storage_ctx = init_storage_proofs(r1cs_buff, wasm_buff, nil)

    echo "storage_ctx: ", storage_ctx.repr
    check storage_ctx != nil

    var
      mpack_arg_path = "tests/proof_test.mpack"
      proofBuff = unsafeBufferFromFile(mpack_arg_path)
    echo "proofArgs:size: ", proofBuff.len()
    let res = prove_mpack_ext(storage_ctx, addr proofBuff)

    echo "result: ", res.repr
    check res != nil
