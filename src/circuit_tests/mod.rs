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
        verify_proof_with_prepared_inputs(&vk, &proof, &public_inputs).unwrap();
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
        verify_proof_with_prepared_inputs(&vk, &proof, &public_inputs).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::CircuitsTests;
    use crate::{poseidon::hash, utils::digest};
    use ruint::aliases::U256;

    #[test]
    fn test_poseidon_hash_circuit() {
        let r1cs = "src/circuit_tests/artifacts/poseidon-hash-test.r1cs";
        let wasm = "src/circuit_tests/artifacts/poseidon-hash-test_js/poseidon-hash-test.wasm";

        let mut hasher = CircuitsTests::new(wasm.to_string(), r1cs.to_string());
        hasher.poseidon_hash(&[U256::from(1)], hash(&[U256::from(1)]));
    }

    #[test]
    fn test_digest_digest_circuit() {
        let r1cs = "src/circuit_tests/artifacts/poseidon-digest-test.r1cs";
        let wasm = "src/circuit_tests/artifacts/poseidon-digest-test_js/poseidon-digest-test.wasm";

        let mut hasher = CircuitsTests::new(wasm.to_string(), r1cs.to_string());
        let input: Vec<U256> = (0..256).map(|_| U256::from(1)).collect();
        hasher.poseidon_digest(&input, digest(&input, Some(16)));
    }
}
