use std::fs::File;

use ark_bn254::{Bn254, Fr};
use ark_circom::{read_zkey, CircomBuilder, CircomConfig};
use ark_groth16::{
    create_random_proof as prove, generate_random_parameters, prepare_verifying_key, verify_proof,
    Proof, ProvingKey,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Read};
use ark_std::rand::rngs::ThreadRng;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ruint::aliases::U256;
use codex_storage_proofs::storage_proofs::{StorageProofs};


// Functions for benchmarking
fn bench_prove(c: &mut Criterion) {
    let wtns = "./witness.wtns";
    let r1cs = "./storer_test.r1cs";
    let zkey = Some("./circuit_0001.zkey".to_string());
    let mut sp = StorageProofs::new(wtns.to_string(), r1cs.to_string(), zkey);
    let chunks: &[U256] = &[];
    let siblings: &[U256] = &[];
    let hashes: &[U256] = &[];
    let path: &[i32] = &[];
    let root = U256::default();
    let salt = U256::default();
    let mut proof_bytes = Vec::new();
    let mut public_inputs_bytes = Vec::new();

    c.bench_function("StorageProofs prove", |b| {
        b.iter(|| {
            black_box(
                sp.prove(
                    chunks,
                    siblings,
                    hashes,
                    path,
                    root,
                    salt,
                    &mut proof_bytes,
                    &mut public_inputs_bytes,
                )
                .unwrap(),
            )
        })
    });
}

fn bench_verify(c: &mut Criterion) {
    let wtns = "./witness.wtns";
    let r1cs = "./storer_test.r1cs";
    let zkey = Some("./circuit_0001.zkey".to_string());
    let mut sp = StorageProofs::new(wtns.to_string(), r1cs.to_string(), zkey);
    let proof_bytes: &[u8] = &[];
    let public_inputs: &[u8] = &[];

    c.bench_function("StorageProofs verify", |b| {
        b.iter(|| {
            black_box(sp.verify(proof_bytes, public_inputs).unwrap());
        })
    });
}

criterion_group!(benches, bench_prove, bench_verify);
criterion_main!(benches);
