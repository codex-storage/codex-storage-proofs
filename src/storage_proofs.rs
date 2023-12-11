use std::fs::File;

use ark_bn254::{Bn254, Fr};
use ark_circom::{read_zkey, CircomBuilder, CircomConfig, CircomCircuit};
use ark_groth16::{
    create_random_proof as prove, generate_random_parameters, prepare_verifying_key, verify_proof,
    Proof, ProvingKey,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Read};
use ark_std::rand::rngs::ThreadRng;
use ruint::aliases::U256;

#[derive(Debug, Clone)]
pub struct StorageProofs {
    builder: CircomBuilder<Bn254>,
    params: ProvingKey<Bn254>,
    rng: ThreadRng,
}

impl StorageProofs {
    // TODO: add rng
    pub fn new(
        wtns: String,
        r1cs: String,
        zkey: Option<String>, /* , rng: Option<ThreadRng> */
    ) -> Self {
        let mut rng = ThreadRng::default();
        let builder = CircomBuilder::new(CircomConfig::<Bn254>::new(wtns, r1cs).unwrap());
        let params: ProvingKey<Bn254> = match zkey {
            Some(zkey) => {
                let mut file = File::open(zkey).unwrap();
                read_zkey(&mut file).unwrap().0
            }
            None => generate_random_parameters::<Bn254, _, _>(builder.setup(), &mut rng).unwrap(),
        };

        Self {
            builder,
            params,
            rng,
        }
    }

    pub fn prove(
        &mut self,
        chunks: &[U256],
        siblings: &[U256],
        hashes: &[U256],
        path: &[i32],
        root: U256,
        salt: U256,
        proof_bytes: &mut Vec<u8>,
        public_inputs_bytes: &mut Vec<u8>,
    ) -> Result<(), String> {
        let mut builder = self.builder.clone();

        // vec of vecs is flattened, since wasm expects a contiguous array in memory
        chunks.iter().for_each(|c| builder.push_input("chunks", *c));

        siblings
            .iter()
            .for_each(|c| builder.push_input("siblings", *c));

        hashes.iter().for_each(|c| builder.push_input("hashes", *c));
        path.iter().for_each(|c| builder.push_input("path", *c));

        builder.push_input("root", root);
        builder.push_input("salt", salt);

        let circuit = builder.build().map_err(|e| e.to_string())?;
        let inputs = circuit
            .get_public_inputs()
            .ok_or("Unable to get public inputs!")?;
        let proof = prove(circuit, &self.params, &mut self.rng).map_err(|e| e.to_string())?;

        proof.serialize(proof_bytes).map_err(|e| e.to_string())?;
        inputs
            .serialize(public_inputs_bytes)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn proof_build_inputs(
        &mut self,
    ) -> Result<CircomCircuit<ark_ec::bn::Bn<ark_bn254::Parameters>>, String> {

        let mut builder = self.builder.clone();

        // vec of vecs is flattened, since wasm expects a contiguous array in memory

        // chunks.iter().for_each(|c| builder.push_input("chunks", *c));

        // siblings
        //     .iter()
        //     .for_each(|c| builder.push_input("siblings", *c));

        // hashes.iter().for_each(|c| builder.push_input("hashes", *c));

        // path.iter().for_each(|c| builder.push_input("path", *c));

        // builder.push_input("root", root);
        // builder.push_input("salt", salt);

        let circuit: CircomCircuit<ark_ec::bn::Bn<ark_bn254::Parameters>> = builder.build()
            .map_err(|e| e.to_string())?;

        Ok(circuit)
    }

    pub fn proof_run(
        &mut self,
        circuit: CircomCircuit<ark_ec::bn::Bn<ark_bn254::Parameters>>,
        proof_bytes: &mut Vec<u8>,
        public_inputs_bytes: &mut Vec<u8>,
    ) -> Result<(), String> {
        let inputs = circuit
            .get_public_inputs()
            .ok_or("Unable to get public inputs!")?;
        let proof =
            prove(circuit, &self.params, &mut self.rng)
            .map_err(|e| e.to_string())?;

        proof
            .serialize(proof_bytes)
            .map_err(|e| e.to_string())?;
        inputs
            .serialize(public_inputs_bytes)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn verify<RR: Read>(
        &mut self,
        proof_bytes: RR,
        mut public_inputs: RR,
    ) -> Result<(), String> {
        let inputs: Vec<Fr> =
            CanonicalDeserialize::deserialize(&mut public_inputs).map_err(|e| e.to_string())?;
        let proof = Proof::<Bn254>::deserialize(proof_bytes).map_err(|e| e.to_string())?;
        let vk = prepare_verifying_key(&self.params.vk);

        verify_proof(&vk, &proof, inputs.as_slice()).map_err(|e| e.to_string())?;

        Ok(())
    }
}
