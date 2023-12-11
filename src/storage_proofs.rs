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

use rmpv;
use rmpv::decode::read_value;

type CircomBuilderParams = CircomBuilder<ark_ec::bn::Bn<ark_bn254::Parameters>>;

const EXT_ID_U256_LE: i8 = 50;
const EXT_ID_U256_BE: i8 = 51;

fn decode_u256(val: &rmpv::Value) -> Result<U256, String> {
    match val {
        rmpv::Value::Ext(id, val) => {
            match *id {
                EXT_ID_U256_LE =>
                    match U256::try_from_le_slice(val) {
                        Some(i) => Ok(i),
                        None => Err("error parsing 256".to_string()),
                    }
                num => return Err(format!("unhandled ext id {}", num)),
            }
        },
        _ => return Err("expected ext mpack kind".to_string()),
    }
}

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
        mut inputs: &[u8]
    ) -> Result<CircomCircuit<ark_ec::bn::Bn<ark_bn254::Parameters>>, String> {

        let values: rmpv::Value = read_value(&mut inputs).map_err(|e| e.to_string())?;
        let args: &Vec<(rmpv::Value, rmpv::Value)> = match values.as_map() {
            Some(args) => args,
            None => return Err("args must be a map of string to arrays".to_string()),
        };

        let mut builder: CircomBuilderParams = self.builder.clone();
        for (key, val) in args {
            let name = match key.as_str() {
                Some(n) => n,
                None => return Err(format!("expected string value")),
            };
            match val {
                rmpv::Value::Array(vals) => {
                    // add a (name, Vec<u256>) or (name, Vev<Vec<u256>>) arrays
                    if vals.len() > 0 && vals[0].is_array() {
                        for inner_val in vals {
                            match inner_val.as_array() {
                                Some(inner_vals) => {
                                    for val in inner_vals {
                                        builder.push_input(name, decode_u256(val)?);
                                    }
                                },
                                _ => return Err("expected inner array of u256".to_string()),
                            }
                        }
                    } else {
                        for val in vals {
                            builder.push_input(name, decode_u256(val)?);
                        }
                    }
                },
                rmpv::Value::String(s) => {
                    // directly add a (name,string) arg pair 
                    // ie, "path" => "/some/file/path"
                    builder.push_input(name, decode_u256(val)?);
                }
                rmpv::Value::Ext(_, _) => {
                    // directly add a (name,u256) arg pair 
                    builder.push_input(name, decode_u256(val)?);
                },
                _ => return Err("unhandled argument kind".to_string()),
            }
        }

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
