use std::fs::File;

use ark_bn254::{Bn254, Fr};
use ark_circom::{read_zkey, CircomBuilder, CircomConfig};
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

    pub fn prove_mpack(
        &mut self,
        proof_bytes: &mut Vec<u8>,
        public_inputs_bytes: &mut Vec<u8>,
    ) -> Result<(), String> {

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

        let circuit = builder.build()
            .map_err(|e| e.to_string())?;
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


#[cfg(test)]
mod tests {
    use ark_std::rand::{distributions::Alphanumeric, rngs::ThreadRng, Rng};
    use rs_poseidon::poseidon::hash;
    use ruint::aliases::U256;

    use crate::circuit_tests::utils::{digest, treehash};

    use rmpv::Value;
    use rmpv::encode::write_value;
    use rmpv::decode::read_value;

    #[test]
    fn test_mpack() {
        let mut buf = Vec::new();
        let val = Value::from("le message");

        // example of serializing the random chunk data
        // we build them up in mpack Value enums
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

        let chunks = data.iter()
            .map(|c| {
                let x = c.0.iter()
                    .map(|c| Value::Ext(50, c.to_le_bytes_vec()))
                    .collect::<Vec<Value>>();
                Value::Array(x)
            })
            .collect::<Vec<Value>>();
        let chunks = Value::Array(chunks);
        let mut data = Value::Map(vec![(Value::String("chunks".into()), chunks.clone() )]);

        println!("Debug: chunks: {:?}", chunks[0][0]);

        // Serialize the value types to an array pointer
        write_value(&mut buf, &data).unwrap();
        let mut rd = &buf[..];
        
        let args = read_value(&mut rd).unwrap();

        assert!(Value::is_map(&args));
        assert!(Value::is_array(&args["chunks"]));
        assert!(Value::is_array(&args["chunks"][0]));
        let mut arg_chunks: Vec<Vec<U256>> = Vec::new();

        // deserialize the data back into u256's
        // instead of this, we'll want to use `builder.push_input`
        args["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .for_each(|c| {
                if let Some(x) = c.as_array() {
                    let mut vals: Vec<U256> = Vec::new();
                    x.iter().for_each(|n| {
                        let b = n.as_ext().unwrap();
                        // ensure it's a LE uin256 which we've set as ext 50
                        assert_eq!(b.0, 50);
                        vals.push(U256::try_from_le_slice(b.1).unwrap());
                        // TODO: change to use
                        // builder.push_input("hashes", *c)
                    });
                    arg_chunks.push(vals);
                } else {
                    panic!("unhandled type!");
                }
            });

        assert_eq!(arg_chunks.len(), 4);
        assert_eq!(arg_chunks[0].len(), 256);

    }
}
