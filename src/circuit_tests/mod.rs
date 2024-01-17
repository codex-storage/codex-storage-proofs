pub mod utils;

#[cfg(test)]
mod test {
    use ark_bn254::Bn254;
    use ark_circom::{CircomBuilder, CircomConfig};
    use ark_groth16::{
        create_random_proof as prove, generate_random_parameters, prepare_inputs,
        prepare_verifying_key, verify_proof_with_prepared_inputs, ProvingKey,
    };
    use ark_std::rand::{distributions::Alphanumeric, rngs::ThreadRng, Rng};
    use rs_poseidon::poseidon::hash;
    use ruint::aliases::U256;

    use crate::{
        circuit_tests::utils::{digest, treehash},
        storage_proofs::StorageProofs,
    };

    pub struct CircuitsTests {
        builder: CircomBuilder<Bn254>,
        params: ProvingKey<Bn254>,
        rng: ThreadRng,
    }

    impl CircuitsTests {
        pub fn new(wtns: String, r1cs: String) -> CircuitsTests {
            let mut rng = ThreadRng::default();
            let builder = CircomBuilder::new(CircomConfig::<Bn254>::new(wtns, r1cs).unwrap());
            let params =
                generate_random_parameters::<Bn254, _, _>(builder.setup(), &mut rng).unwrap();

            CircuitsTests {
                builder,
                params,
                rng,
            }
        }

        pub fn poseidon_hash(&mut self, elements: &[U256], hash: U256) -> bool {
            let mut builder = self.builder.clone();

            elements.iter().for_each(|c| builder.push_input("in", *c));
            builder.push_input("hash", hash);

            let circuit = builder.build().unwrap();
            let inputs = circuit.get_public_inputs().unwrap();
            let proof = prove(circuit, &self.params, &mut self.rng).unwrap();
            let vk = prepare_verifying_key(&self.params.vk);
            let public_inputs = prepare_inputs(&vk, &inputs).unwrap();
            verify_proof_with_prepared_inputs(&vk, &proof, &public_inputs).is_ok()
        }

        pub fn poseidon_digest(&mut self, elements: &[U256], hash: U256) -> bool {
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

            verify_proof_with_prepared_inputs(&vk, &proof, &public_inputs).is_ok()
        }
    }

    #[test]
    fn test_poseidon_hash() {
        let r1cs = "./src/circuit_tests/artifacts/poseidon-hash-test.r1cs";
        let wasm = "./src/circuit_tests/artifacts/poseidon-hash-test_js/poseidon-hash-test.wasm";

        let mut hasher = CircuitsTests::new(wasm.to_string(), r1cs.to_string());
        assert!(hasher.poseidon_hash(&[U256::from(1)], hash(&[U256::from(1)])));
    }

    #[test]
    fn test_poseidon_digest() {
        let r1cs = "./src/circuit_tests/artifacts/poseidon-digest-test.r1cs";
        let wasm =
            "./src/circuit_tests/artifacts/poseidon-digest-test_js/poseidon-digest-test.wasm";

        let mut hasher = CircuitsTests::new(wasm.to_string(), r1cs.to_string());
        let input: Vec<U256> = (0..256).map(|c| U256::from(c)).collect();
        assert!(hasher.poseidon_digest(&input, digest(&input, Some(16))));
    }

    #[test]
    fn test_storer() {
        let r1cs = "./src/circuit_tests/artifacts/storer-test.r1cs";
        let wasm = "./src/circuit_tests/artifacts/storer-test_js/storer-test.wasm";
        let mut prover = StorageProofs::new(wasm.to_string(), r1cs.to_string(), None);

        // generate a tuple of (preimages, hash), where preimages is a vector of 256 U256s
        // and hash is the hash of each vector generated using the digest function
        let data = (0..4)
            .map(|_| {
                let rng = ThreadRng::default();
                let preimages: Vec<U256> = rng
                    .sample_iter(Alphanumeric)
                    .take(256)
                    .map(|c| U256::from(c))
                    .collect();
                let hash = digest(&preimages, Some(16));
                (preimages, hash)
            })
            .collect::<Vec<(Vec<U256>, U256)>>();

        let chunks: Vec<U256> = data.iter().flat_map(|c| c.0.to_vec()).collect();
        let hashes: Vec<U256> = data.iter().map(|c| c.1).collect();
        let path = [0, 1, 2, 3].to_vec();

        let parent_hash_l = hash(&[hashes[0], hashes[1]]);
        let parent_hash_r = hash(&[hashes[2], hashes[3]]);

        let siblings = &[
            hashes[1],
            parent_hash_r,
            hashes[0],
            parent_hash_r,
            hashes[3],
            parent_hash_l,
            hashes[2],
            parent_hash_l,
        ];

        let root = treehash(hashes.as_slice());
        // let proof_bytes = &mut Vec::new();
        // let public_inputs_bytes = &mut Vec::new();

        // prover
        //     .prove(
        //         chunks.as_slice(),
        //         siblings,
        //         hashes.as_slice(),
        //         path.as_slice(),
        //         root,
        //         root, // random salt - block hash
        //         proof_bytes,
        //         public_inputs_bytes,
        //     )
        //     .unwrap();

        // assert!(prover
        //     .verify(proof_bytes.as_slice(), public_inputs_bytes.as_slice())
        //     .is_ok());
    }
}
