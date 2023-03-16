use crate::storageproofs::StorageProofs;
use std::str;

#[no_mangle]
pub extern "C" fn init(
    r1cs: *const u8,
    r1cs_len: usize,
    wasm: *const u8,
    wasm_len: usize,
) -> *mut StorageProofs {
    let r1cs = unsafe {
        let slice = std::slice::from_raw_parts(r1cs, r1cs_len);
        str::from_utf8(slice).unwrap()
    };

    let wasm = unsafe {
        let slice = std::slice::from_raw_parts(wasm, wasm_len);
        str::from_utf8(slice).unwrap()
    };

    let storage_proofs = Box::into_raw(Box::new(StorageProofs::new(
        wasm.to_string(),
        r1cs.to_string(),
    )));

    return storage_proofs;
}

#[cfg(test)]
mod tests {
    use super::init;

    #[test]
    fn should_prove() {
        let r1cs = "/Users/dryajov/personal/projects/status/codex-zk/test/circuits/artifacts/storer_test.r1cs";
        let wasm = "/Users/dryajov/personal/projects/status/codex-zk/test/circuits/artifacts/storer_test_js/storer_test.wasm";

        let prover = init(r1cs.as_ptr(), r1cs.len(), wasm.as_ptr(), wasm.len());
    }
}
