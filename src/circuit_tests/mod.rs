mod utils;

use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomConfig};
use ark_groth16::{
    create_random_proof as prove, generate_random_parameters, prepare_inputs,
    prepare_verifying_key, verify_proof_with_prepared_inputs, ProvingKey,
};
use ark_std::rand::rngs::ThreadRng;
use ruint::aliases::U256;

pub struct CircuitsTests {
    builder: CircomBuilder<Bn254>,
    params: ProvingKey<Bn254>,
    rng: ThreadRng,
}

impl CircuitsTests {
    pub fn new(wtns: String, r1cs: String) -> CircuitsTests {
        let mut rng = ThreadRng::default();
        let builder = CircomBuilder::new(CircomConfig::<Bn254>::new(wtns, r1cs).unwrap());
        let params = generate_random_parameters::<Bn254, _, _>(builder.setup(), &mut rng).unwrap();

        CircuitsTests {
            builder,
            params,
            rng,
        }
    }

    pub fn poseidon_hash(&mut self, elements: &[U256], hash: U256) {
        let mut builder = self.builder.clone();

        elements.iter().for_each(|c| builder.push_input("in", *c));
        builder.push_input("hash", hash);

        let circuit = builder.build().unwrap();
        let inputs = circuit.get_public_inputs().unwrap();
        let proof = prove(circuit, &self.params, &mut self.rng).unwrap();
        let vk = prepare_verifying_key(&self.params.vk);
        let public_inputs = prepare_inputs(&vk, &inputs).unwrap();

        assert!(verify_proof_with_prepared_inputs(&vk, &proof, &public_inputs).is_ok());
    }

    pub fn poseidon_digest(&mut self, elements: &[U256], hash: U256) {
        let mut builder = self.builder.clone();

        elements
            .iter()
            .for_each(|c| builder.push_input("block", *c));
        builder.push_input("hash", hash);

        let circuit = builder.build().unwrap();
        let inputs = circuit.get_public_inputs().unwrap();
        let proof = prove(circuit, &self.params, &mut self.rng).unwrap();
        let vk = prepare_verifying_key(&self.params.vk);
        let public_inputs = prepare_inputs(&vk, &inputs).unwrap();

        assert!(verify_proof_with_prepared_inputs(&vk, &proof, &public_inputs).is_ok());
    }
}

#[cfg(test)]
mod test {
    use super::CircuitsTests;
    use crate::{
        circuit_tests::utils::digest, circuit_tests::utils::merkelize, poseidon::hash,
        storageproofs::StorageProofs,
    };
    use ruint::aliases::U256;

    #[test]
    fn test_poseidon_hash() {
        let r1cs = "./src/circuit_tests/artifacts/poseidon-hash-test.r1cs";
        let wasm = "./src/circuit_tests/artifacts/poseidon-hash-test_js/poseidon-hash-test.wasm";

        let mut hasher = CircuitsTests::new(wasm.to_string(), r1cs.to_string());
        hasher.poseidon_hash(&[U256::from(1)], hash(&[U256::from(1)]));
    }

    #[test]
    fn test_poseidon_digest() {
        let r1cs = "./src/circuit_tests/artifacts/poseidon-digest-test.r1cs";
        let wasm =
            "./src/circuit_tests/artifacts/poseidon-digest-test_js/poseidon-digest-test.wasm";

        let mut hasher = CircuitsTests::new(wasm.to_string(), r1cs.to_string());
        let input: Vec<U256> = (0..256).map(|_| U256::from(1)).collect();
        hasher.poseidon_digest(&input, digest(&input, Some(16)));
    }

    #[test]
    fn test_storer() {
        // generate a tuple of (preimages, hash), where preimages is a vector of 256 U256s
        // and hash is the hash of each vector generated using the digest function
        let data = (0..4)
            .map(|_| {
                let preimages: Vec<U256> = (0..256).map(|_| U256::from(1)).collect();
                let hash = digest(&preimages, Some(16));
                (preimages, hash)
            })
            .collect::<Vec<(Vec<U256>, U256)>>();

        let chunks: Vec<Vec<U256>> = data.iter().map(|c| c.0.to_vec()).collect();
        let hashes: Vec<U256> = data.iter().map(|c| c.1).collect();
        let path = [0, 1, 2, 3].to_vec();

        let parent_hash_l = hash(&[hashes[0], hashes[1]]);
        let parent_hash_r = hash(&[hashes[2], hashes[3]]);

        let siblings = [
            [hashes[1], parent_hash_r].to_vec(),
            [hashes[1], parent_hash_r].to_vec(),
            [hashes[3], parent_hash_l].to_vec(),
            [hashes[2], parent_hash_l].to_vec(),
        ]
        .to_vec();

        let r1cs = "./src/circuit_tests/artifacts/storer-test.r1cs";
        let wasm = "./src/circuit_tests/artifacts/storer-test_js/storer-test.wasm";
        let mut prover = StorageProofs::new(wasm.to_string(), r1cs.to_string());

        let root = merkelize(hashes.as_slice());
        let proof_bytes = &mut Vec::new();
        let public_inputs_bytes = &mut Vec::new();

        prover
            .prove(
                chunks,
                siblings,
                hashes,
                path,
                root,
                root, // random salt - block hash
                proof_bytes,
                public_inputs_bytes,
            )
            .unwrap();

        assert!(prover
            .verify(proof_bytes.as_slice(), public_inputs_bytes.as_slice())
            .is_ok());
    }
}
