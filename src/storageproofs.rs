use ark_bn254::{Bn254, Fr};
use ark_circom::{CircomBuilder, CircomConfig};
use ark_groth16::{
    create_random_proof as prove, generate_random_parameters, prepare_verifying_key, verify_proof,
    Proof, ProvingKey,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Read};
use ark_std::rand::rngs::ThreadRng;
use ruint::aliases::U256;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct StorageProofs {
    builder: CircomBuilder<Bn254>,
    pvk: ProvingKey<Bn254>,
    rng: ThreadRng,
}

impl StorageProofs {
    pub fn new(wtns: String, r1cs: String) -> Self {
        let mut rng = ThreadRng::default();
        let builder = CircomBuilder::new(CircomConfig::<Bn254>::new(wtns, r1cs).unwrap());
        let pvk = generate_random_parameters::<Bn254, _, _>(builder.setup(), &mut rng).unwrap();

        Self { builder, pvk, rng }
    }

    pub fn prove(
        &mut self,
        chunks: Vec<Vec<U256>>,
        siblings: Vec<Vec<U256>>,
        hashes: Vec<U256>,
        path: Vec<u32>,
        root: U256,
        salt: U256,
        proof_bytes: &mut Vec<u8>,
        public_inputs_bytes: &mut Vec<u8>,
    ) -> Result<(), String> {
        let mut builder = self.builder.clone();

        // vec of vecs is flattened, since wasm expects a contiguous array in memory
        chunks
            .iter()
            .flat_map(|c| c.into_iter())
            .for_each(|c| builder.push_input("chunks", *c));

        siblings
            .iter()
            .flat_map(|c| c.into_iter())
            .for_each(|c| builder.push_input("siblings", *c));

        hashes.iter().for_each(|c| builder.push_input("hashes", *c));

        path.iter().for_each(|c| builder.push_input("path", *c));

        builder.push_input("root", root);

        builder.push_input("salt", salt);

        let circuit = builder.build().map_err(|e| e.to_string())?;
        let inputs = circuit
            .get_public_inputs()
            .ok_or("Unable to get public inputs!")?;
        let proof = prove(circuit, &self.pvk, &mut self.rng).map_err(|e| e.to_string())?;

        proof.serialize(proof_bytes).map_err(|e| e.to_string())?;
        inputs
            .serialize(public_inputs_bytes)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn verify<R: Read>(self, proof_bytes: R, mut public_inputs: R) -> Result<(), String> {
        let inputs: Vec<Fr> =
            CanonicalDeserialize::deserialize(&mut public_inputs).map_err(|e| e.to_string())?;
        let proof = Proof::<Bn254>::deserialize(proof_bytes).map_err(|e| e.to_string())?;
        let vk = prepare_verifying_key(&self.pvk.vk);

        verify_proof(&vk, &proof, &inputs.as_slice()).map_err(|e| e.to_string())?;

        Ok(())
    }
}
