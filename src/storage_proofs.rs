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

type Params256Ty = ark_ec::bn::Bn<ark_bn254::Parameters>;

pub const EXT_ID_U256_LE: i8 = 50;
pub const EXT_ID_U256_BE: i8 = 51;


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

    pub fn prove_mpack(
        &mut self,
        inputs: &[u8],
        proof_bytes: &mut Vec<u8>,
        public_inputs_bytes: &mut Vec<u8>,
    ) -> Result<(), String> {
        let mut builder: CircomBuilder<Params256Ty> = self.builder.clone();

        parse_mpack_args(&mut builder, inputs)?;
        // pub fn prove(
        //     &mut self,
        //     chunks: &[U256],
        //     siblings: &[U256],
        //     hashes: &[U256],
        //     path: &[i32],
        //     root: U256,
        //     salt: U256,
        //     proof_bytes: &mut Vec<u8>,
        //     public_inputs_bytes: &mut Vec<u8>,

        let circuit: CircomCircuit<Params256Ty> = builder.build()
            .map_err(|e| e.to_string())?;

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

fn parse_mpack_args(builder: &mut CircomBuilder<Params256Ty>,
                    mut inputs: &[u8]) -> Result<(), String> {
    let values: rmpv::Value = read_value(&mut inputs).map_err(|e| e.to_string())?;
    let args: &Vec<(rmpv::Value, rmpv::Value)> = match values.as_map() {
        Some(args) => args,
        None => return Err("args must be a map of string to arrays".to_string()),
    };

    for (key, val) in args {
        let name = match key.as_str() {
            Some(n) => n,
            None => return Err(format!("expected string value")),
        };
        match val {
            // add a (name, Vec<u256>) or (name, Vev<Vec<u256>>) arrays
            rmpv::Value::Array(vals) => {
                println!("deserde: array: {} size: {}", name, vals.len());
                if vals.len() > 0 && vals[0].is_array() {
                    println!("deserde: arrayOfArrays: {}", name);
                    for inner_val in vals {
                        match inner_val.as_array() {
                            Some(inner_vals) => {
                                println!("\tinner array: {} sz: {}", name, inner_vals.len());
                                for val in inner_vals {
                                    let n = decode_u256(val)?;
                                    println!("\tval: {} ", n);
                                    // builder.push_input(name, n);
                                }
                            },
                            _ => {
                                print!("error expected array: {}", name);
                                return Err("expected inner array of u256".to_string())
                            },
                        }
                    }
                } else {
                    println!("deserde: name: {}", name);
                    for val in vals {
                        let n = decode_u256(val)?;
                        println!("\t{}", n);
                        builder.push_input(name, n);
                    }
                    println!("done: name: {}", name);
                }
            },
            // directly add a (name,u256) arg pair 
            rmpv::Value::Ext(_, _) => {
                let n = decode_u256(val)?;
                println!("deserde: name: {} u256: {}", name, n);
                builder.push_input(name, n);
            },
            _ => return Err("unhandled argument kind".to_string()),
        }
    }

    println!("parse_mpack_args DONE!");
    Ok(())
}
