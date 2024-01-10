
import unittest2
import codex_storage_proofs

suite "storage proofs ffi":
  test "basic ffi circuit":
    var
      r1cs_path = "src/circuit_tests/artifacts/storer-test.r1cs"
      wasm_path = "src/circuit_tests/artifacts/storer-test_js/storer-test.wasm"

    let
      r1cs_buff = unsafeBufferPath(r1cs_path)
      wasm_buff = unsafeBufferPath(wasm_path)

    let storage_ctx = init_storage_proofs(r1cs_buff, wasm_buff, nil)

    echo "storage_ctx: ", storage_ctx.repr
    check storage_ctx != nil

    var
      mpack_arg_path = "test/proof_test.mpack"
      proofBuff = unsafeBufferFromFile(mpack_arg_path)
    echo "proofArgs:size: ", proofBuff.len()
    let res = prove_mpack_ext(storage_ctx, addr proofBuff)

    echo "result: ", res.repr
    check res != nil
