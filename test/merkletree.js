//import { generateMerkleProof, generateMerkleTree } from "@zk-kit/protocols"
import { IncrementalMerkleTree } from "@zk-kit/incremental-merkle-tree"
import { poseidon } from "circomlibjs"
import assert from "assert";
import path from "path";
import * as snarkjs from "snarkjs";

describe("MerkleTree", function ()  {

  const depth = 21
  const numberOfLeaves = 2 ** 2
  const hashFn = poseidon
  const zeroValue = BigInt(0)
  const arity = 2
  const queryLen = 1

  const r1csPath = path.join("circuits", "storer.r1cs");
  const wasmPath = path.join("circuits", "storer_js", "storer.wasm");

  var tree
  it ("generate Merkle Tree from data", async () => {
    tree = new IncrementalMerkleTree(hashFn, depth, zeroValue, arity)
    for (let i = 0; i < numberOfLeaves; i += 1) {
      tree.insert(BigInt(i + 1))
    }
  })

  var merkleProof
  it ("create proof", async () => {
    merkleProof = tree.createProof(0)
  })

  it ("verify proof", async () => {
    assert(tree.verifyProof(merkleProof))
    console.warn(merkleProof)
  })

  const wtns = {type: "mem"};
  it ("witness calculate", async () => {

    // inputs defined in circuit:
    //   signal input chunks[qLen][blockSize];
    //   signal input chunkHashes[qLen];
    //   signal input indices[qLen];
    //   signal input treeSiblings[qLen][nLevels];
    //   signal input root;

    console.warn(hashFn([1]))
    let chunks = [[1,2]]
    await snarkjs.wtns.calculate(
      { chunks: chunks,
        chunkHashes: [hashFn(chunks[0])],
        indices: [0],
        treeSiblings: [merkleProof.siblings.slice(1)],
        root: merkleProof.root
      }, wasmPath, wtns);
    console.warn("witness: ", wtns);
    // console.warn("witness: ", wtns.data.length, " bytes");
    // console.warn("witness: ", sizeOf(wtns.data), " bytes");
  })
})
