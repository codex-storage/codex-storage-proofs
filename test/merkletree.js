import { IncrementalMerkleTree } from "@zk-kit/incremental-merkle-tree"
import { poseidon } from "circomlibjs"
import assert from "assert";
import path from "path";
import * as snarkjs from "snarkjs";
import wasm from "circom_tester";
import { buildBn128, buildBls12381} from "ffjavascript";

describe("MerkleTree", function ()  {
  this.timeout(1000000000);

  // these parameters should match the circuit
  const depth = 21  // depth of full tree from data: sum of block hash depth and dataset treehash depth 
  const hashFn = poseidon
  const zeroValue = BigInt(0) // padding symbol in the Merkle Tree
  const arity = 2 // 
  const queryLen = 1

  const numberOfLeaves = 2 ** 7 // example dataset size

  const circuitPath = path.join("circuits", "storer.circom");
  const r1csPath = path.join("circuits", "storer.r1cs");
  const wasmPath = path.join("circuits", "storer_js", "storer.wasm");


  let curve;
  const ptau_0 = {type: "mem"};
  const ptau_1 = {type: "mem"};
  const ptau_2 = {type: "mem"};
  const ptau_beacon = {type: "mem"};
  const ptau_final = {type: "mem"};
  const ptau_challenge2 = {type: "mem"};
  const ptau_response2 = {type: "mem"};
  const zkey_0 = {type: "mem"};
  const zkey_1 = {type: "mem"};
  const zkey_2 = {type: "mem"};
  const zkey_final = {type: "mem"};
  const zkey_plonk = {type: "mem"};
  const bellman_1 = {type: "mem"};
  const bellman_2 = {type: "mem"};
  let vKey;
  let vKeyPlonk;
  const wtns = {type: "mem"};
  let proof;
  let publicSignals;

  before( async () => {
    curve = await buildBn128();
  });
  after( async () => {
    await curve.terminate();
  });


  // create Merkle Tree for example dataset
  let tree
  it ("generate Merkle Tree from data", () => {
    tree = new IncrementalMerkleTree(hashFn, depth, zeroValue, arity)
    for (let i = 0; i < numberOfLeaves; i += 1) {
      tree.insert(BigInt(i + 1))
    }
  })

  const index = 0
  // Create an example Merkle Proof
  let merkleProof
  it ("create Merkle proof", () => {
    merkleProof = tree.createProof(index)
  })

  // Verify the above proof just to be on the safe side
  it ("verify Merkle proof", () => {
    assert(tree.verifyProof(merkleProof))
    // console.warn(merkleProof)
  })

  let cir
  it ("compile circuit", async () => {
    cir = await wasm.wasm(circuitPath)
    // console.warn(cir)
  })

  const chunks = [[1,2]]
  let circuitInputs
  it ("witness calculate", async () => {

    // inputs defined in circuit:
    //   signal input chunks[qLen][blockSize];
    //   signal input chunkHashes[qLen];
    //   signal input indices[qLen];
    //   signal input treeSiblings[qLen][nLevels];
    //   signal input root;

    circuitInputs = {
      chunks: chunks,
      chunkHashes: [hashFn(chunks[index])],
      indices: [index],
      treeSiblings: [merkleProof.siblings.slice(1)],
      root: merkleProof.root
    }
    await snarkjs.wtns.calculate(circuitInputs, wasmPath, wtns);
    // await cir.calculateWitness(circuitInputs, true);
    // console.warn("witness: ", wtns);
    // console.warn("witness: ", wtns.data.length, " bytes");
    // console.warn("witness: ", sizeOf(wtns.data), " bytes");
  })

  // set ceremony size
  // The second parameter is the power of two of the maximum number of constraints that the ceremony can accept: in this case, the number of constraints is 2 ^ 12 = 4096. The maximum value supported here is 28, which means you can use snarkjs to securely generate zk-snark parameters for circuits with up to 2 ^ 28 (≈268 million) constraints.
  // see https://github.com/iden3/snarkjs/blob/master/README.md#1-start-a-new-powers-of-tau-ceremony
  const power = 13
  
  it ("powersoftau new", async () => {
    await snarkjs.powersOfTau.newAccumulator(curve, power, ptau_0);
  });

  it ("powersoftau contribute ", async () => {
    await snarkjs.powersOfTau.contribute(ptau_0, ptau_1, "C1", "Entropy1");
  });

  it ("powersoftau export challenge", async () => {
    await snarkjs.powersOfTau.exportChallenge(ptau_1, ptau_challenge2);
  });

  it ("powersoftau challenge contribute", async () => {
    await snarkjs.powersOfTau.challengeContribute(curve, ptau_challenge2, ptau_response2, "Entropy2");
  });

  it ("powersoftau import response", async () => {
    await snarkjs.powersOfTau.importResponse(ptau_1, ptau_response2, ptau_2, "C2", true);
  });

  it ("powersoftau beacon", async () => {
    await snarkjs.powersOfTau.beacon(ptau_2, ptau_beacon, "B3", "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20", 10);
  });

  it ("powersoftau prepare phase2", async () => {
    await snarkjs.powersOfTau.preparePhase2(ptau_beacon, ptau_final);
  });

  it ("powersoftau verify", async () => {
    const res = await snarkjs.powersOfTau.verify(ptau_final);
    assert(res);
  });

  it ("groth16 setup", async () => {
    await snarkjs.zKey.newZKey(r1csPath, ptau_final, zkey_0);
    console.warn(zkey_0);
  });

  it ("zkey contribute ", async () => {
    await snarkjs.zKey.contribute(zkey_0, zkey_1, "p2_C1", "pa_Entropy1");
  });

  it ("zkey export bellman", async () => {
    await snarkjs.zKey.exportBellman(zkey_1, bellman_1);
  });

  it ("zkey bellman contribute", async () => {
    await snarkjs.zKey.bellmanContribute(curve, bellman_1, bellman_2, "pa_Entropy2");
  });

  it ("zkey import bellman", async () => {
    await snarkjs.zKey.importBellman(zkey_1, bellman_2, zkey_2, "C2");
  });

  it ("zkey beacon", async () => {
    await snarkjs.zKey.beacon(zkey_2, zkey_final, "B3", "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20", 10);
  });

  it ("zkey verify r1cs", async () => {
    const res = await snarkjs.zKey.verifyFromR1cs(r1csPath, ptau_final, zkey_final);
    assert(res);
  });

  it ("zkey verify init", async () => {
    const res = await snarkjs.zKey.verifyFromInit(zkey_0, ptau_final, zkey_final);
    assert(res);
  });

  it ("zkey export verificationkey", async () => {
    vKey = await snarkjs.zKey.exportVerificationKey(zkey_final);
  });

  it ("witness calculate", async () => {
    await snarkjs.wtns.calculate(circuitInputs, wasmPath, wtns);
  });

  it ("groth16 proof", async () => {
    const res = await snarkjs.groth16.prove(zkey_final, wtns);
    proof = res.proof;
    publicSignals = res.publicSignals;
  });


  it ("groth16 verify", async () => {
    const res = await snarkjs.groth16.verify(vKey, publicSignals, proof);
    assert(res == true);
  });

  it ("plonk setup", async () => {
    await snarkjs.plonk.setup(r1csPath, ptau_final, zkey_plonk);
  });

  it ("zkey export verificationkey", async () => {
    vKey = await snarkjs.zKey.exportVerificationKey(zkey_plonk);
  });

  it ("plonk proof", async () => {
    const res = await snarkjs.plonk.prove(zkey_plonk, wtns);
    proof = res.proof;
    publicSignals = res.publicSignals;
    console.warn("proof: ", proof, " bytes");
    console.warn("public: ", publicSignals, " bytes");
  });


  it ("plonk verify", async () => {
    const res = await snarkjs.plonk.verify(vKey, publicSignals, proof);
    assert(res == true);
  });

})
