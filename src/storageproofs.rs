// use ark_bn254::Bn254;
// use ark_std::rand::rngs::ThreadRng;
// use ark_circom::{CircomBuilder, CircomConfig};
// use ark_groth16::{ProvingKey, generate_random_parameters, prepare_verifying_key, create_random_proof as prove};
// use ruint::aliases::U256;
// use crate::poseidon::hash1;

// #[derive(Debug, Clone)]
// #[repr(C)]
// pub struct StorageProofs {
//     builder: CircomBuilder<Bn254>,
//     pvk: ProvingKey<Bn254>,
//     rng: ThreadRng,
// }

// impl StorageProofs {
//     pub fn new(wtns: String, r1cs: String) -> Self {
//         let mut rng = ThreadRng::default();
//         let builder = CircomBuilder::new(CircomConfig::<Bn254>::new(wtns, r1cs).unwrap());
//         let pvk = generate_random_parameters::<Bn254, _, _>(builder.setup(), &mut rng).unwrap();

//         Self { builder, pvk, rng }
//     }

//     pub fn prove(
//         &mut self,
//         chunks: Vec<Vec<Fq>>,
//         siblings: Vec<Vec<Fq>>,
//         hashes: Vec<Fq>,
//         path: Vec<u32>,
//         root: Fq,
//         salt: Fq,
//         proof_bytes: Vec<u8>,
//         public_inputs_bytes: Vec<u8>,
//     ) -> Result<(), String> {
//         let mut builder = self.builder.clone();

//         chunks.iter().flat_map(|c| c.into_iter()).for_each(|c| {
//             builder.push_input(
//                 "chunks",
//                 BigInt::from_biguint(Sign::Plus, c.into_repr().into()),
//             )
//         });

//         siblings.iter().flat_map(|c| c.into_iter()).for_each(|c| {
//             builder.push_input(
//                 "siblings",
//                 BigInt::from_biguint(Sign::Plus, c.into_repr().into()),
//             )
//         });

//         hashes.iter().for_each(|c| {
//             builder.push_input(
//                 "hashes",
//                 BigInt::from_biguint(Sign::Plus, c.into_repr().into()),
//             )
//         });

//         path.iter()
//             .for_each(|c| builder.push_input("path", BigInt::new(Sign::Plus, vec![*c])));

//         builder.push_input(
//             "root",
//             BigInt::from_biguint(Sign::Plus, root.into_repr().into()),
//         );

//         builder.push_input(
//             "salt",
//             BigInt::from_biguint(Sign::Plus, salt.into_repr().into()),
//         );

//         let circuit = builder.build().unwrap();
//         let inputs = circuit.get_public_inputs().unwrap();
//         let proof = prove(circuit, &self.pvk, &mut self.rng).unwrap();
//         let vk = prepare_verifying_key(&self.pvk.vk);

//         // proof.serialize(proof_bytes).unwrap();
//         // inputs.serialize(public_inputs_bytes).unwrap();

//         Ok(())
//     }

//     // fn verify<R: Read>(self, hashes: Vec<i32>, root: i32, salt: i32,vk_bytes: R, proof_bytes: R) -> Result<(), String> {
//     //     let vk = ProvingKey::<Bn254>::deserialize(vk_bytes).unwrap();
//     //     let proof = Proof::<Bn254>::deserialize(proof_bytes).unwrap();

//     //     let vk = prepare_verifying_key(&self.pvk.vk);
//     //     verify_proof(&vk, &proof, &public_inputs).unwrap();

//     //     Ok(())
//     // }
// }

// #[cfg(test)]
// mod test {
//     use super::StorageProofs;
//     use ark_bn254::Fq;
//     use ark_ff::{UniformRand, Zero};
//     use ark_std::rand::{rngs::ThreadRng, Rng};
//     use arkworks_native_gadgets::{
//         poseidon::{sbox::PoseidonSbox, *},
//         prelude::ark_ff::PrimeField,
//     };

//     use arkworks_utils::{
//         bytes_matrix_to_f, bytes_vec_to_f, poseidon_params::setup_poseidon_params, Curve,
//     };

//     fn digest(input: Vec<Fq>, chunk_size: Option<usize>) -> Result<Fq, PoseidonError> {
//         let chunk_size = chunk_size.unwrap_or(4);
//         let chunks = ((input.len() as f32) / (chunk_size as f32)).ceil() as usize;
//         let mut concat = vec![];
//         let hasher = hash1(Curve::Bn254, 5, (chunk_size + 1) as u8);

//         let mut i: usize = 0;
//         while i < chunks {
//             let range = (i * chunk_size)..std::cmp::min((i + 1) * chunk_size, input.len());

//             let mut chunk: Vec<Fq> = input[range].to_vec();

//             if chunk.len() < chunk_size {
//                 chunk.resize(chunk_size as usize, Fq::zero());
//             }

//             concat.push(hasher(chunk)?);
//             i += chunk_size;
//         }

//         if concat.len() > 1 {
//             return hasher(concat);
//         }

//         return Ok(concat[0]);
//     }

//     fn merkelize(leafs: Vec<Fq>) -> Fq {
//         // simple merkle root (treehash) generator
//         // unbalanced trees will have the last leaf duplicated
//         let mut merkle: Vec<Fq> = leafs;
//         let hasher = hasher(Curve::Bn254, 5, 3);

//         while merkle.len() > 1 {
//             let mut new_merkle = Vec::new();
//             let mut i = 0;
//             while i < merkle.len() {
//                 new_merkle.push(hasher(vec![merkle[i], merkle[i + 1]]).unwrap());
//                 i += 2;
//             }

//             if merkle.len() % 2 == 1 {
//                 new_merkle.push(
//                     hasher(vec![merkle[merkle.len() - 2], merkle[merkle.len() - 2]]).unwrap(),
//                 );
//             }

//             merkle = new_merkle;
//         }

//         return merkle[0];
//     }

//     #[test]
//     fn should_proove() {
//         let mut rng = ThreadRng::default();
//         let data: Vec<(Vec<Fq>, Fq)> = (0..4)
//             .map(|_| {
//                 let preimages = vec![Fq::rand(&mut rng); 32];
//                 let hash = digest(preimages.clone(), None).unwrap();
//                 return (preimages, hash);
//             })
//             .collect();

//         let chunks: Vec<Vec<Fq>> = data.iter().map(|c| c.0.to_vec()).collect();
//         let hashes: Vec<Fq> = data.iter().map(|c| c.1).collect();
//         let path = [0, 1, 2, 3].to_vec();

//         let hash2 = hasher(Curve::Bn254, 5, 3);
//         let parent_hash_l = hash2(vec![hashes[0], hashes[1]]).unwrap();
//         let parent_hash_r = hash2(vec![hashes[2], hashes[3]]).unwrap();

//         let siblings = [
//             [hashes[1], parent_hash_r].to_vec(),
//             [hashes[1], parent_hash_r].to_vec(),
//             [hashes[3], parent_hash_l].to_vec(),
//             [hashes[2], parent_hash_l].to_vec(),
//         ]
//         .to_vec();

//         let root = merkelize(hashes.clone());
//         let mut proof_bytes: Vec<u8> = Vec::new();
//         let mut public_inputs_bytes: Vec<u8> = Vec::new();

//         let r1cs = "/Users/dryajov/personal/projects/status/codex-zk/test/circuits/artifacts/storer_test.r1cs";
//         let wasm = "/Users/dryajov/personal/projects/status/codex-zk/test/circuits/artifacts/storer_test_js/storer_test.wasm";

//         let mut prover = StorageProofs::new(wasm.to_string(), r1cs.to_string());
//         prover
//             .prove(
//                 chunks,
//                 siblings,
//                 hashes,
//                 path,
//                 root,
//                 root, // random salt
//                 proof_bytes,
//                 public_inputs_bytes,
//             )
//             .unwrap();
//     }
// }
