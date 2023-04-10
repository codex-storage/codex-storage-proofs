const chai = require("chai");
const path = require("path");
const crypto = require("crypto");
const F1Field = require("ffjavascript").F1Field;
const Scalar = require("ffjavascript").Scalar;
const {c} = require("circom_tester");
const chaiAsPromised = require('chai-as-promised');
const poseidon = require("circomlibjs/src/poseidon");
const wasm_tester = require("circom_tester").wasm;
// const snarkjs = require("snarkjs");
// const fs = require("fs");

chai.use(chaiAsPromised);

const p = Scalar.fromString("21888242871839275222246405745257275088548364400416034343698204186575808495617");
const Fr = new F1Field(p);

const assert = chai.assert;
const expect = chai.expect;

function digest(input, chunkSize = 5) {
  let chunks = Math.ceil(input.length / chunkSize);
  let concat = [];

  for (let i = 0; i < chunks; i++) {
    let chunk = input.slice(i * chunkSize, (i + 1) * chunkSize);
    if (chunk.length < chunkSize) {
      chunk = chunk.concat(Array(chunkSize - chunk.length).fill(0));
    }
    concat.push(poseidon(chunk));
  }

  if (concat.length > 1) {
    return poseidon(concat);
  }

  return concat[0]
}

function merkelize(leafs) {
  // simple merkle root (treehash) generator
  // unbalanced trees will have the last leaf duplicated
  var merkle = leafs;

  while (merkle.length > 1) {
    var newMerkle = [];

    var i = 0;
    while (i < merkle.length) {
      newMerkle.push(digest([merkle[i], merkle[i + 1]], 2));
      i += 2;
    }

    if (merkle.length % 2 == 1) {
      newMerkle.add(digest([merkle[merkle.length - 2], merkle[merkle.length - 2]], 2));
    }

    merkle = newMerkle;
  }

  return merkle[0];
}

describe("Storer test", function () {
  this.timeout(100000);

  const a = Array.from(crypto.randomBytes(256).values()).map((v) => BigInt(v));
  const aHash = digest(a, 16);
  const b = Array.from(crypto.randomBytes(256).values()).map((v) => BigInt(v));
  const bHash = digest(b, 16);
  const c = Array.from(crypto.randomBytes(256).values()).map((v) => BigInt(v));
  const cHash = digest(c, 16);
  const d = Array.from(crypto.randomBytes(256).values()).map((v) => BigInt(v));
  const dHash = digest(d, 16);
  const salt = Array.from(crypto.randomBytes(256).values()).map((v) => BigInt(v));
  const saltHash = digest(salt, 16);

  it("Should merkelize", async () => {
    let root = merkelize([aHash, bHash]);
    let hash = digest([aHash, bHash], 2);

    assert.equal(hash, root);
  });

  it("Should verify chunk is correct and part of dataset", async () => {
    const cir = await wasm_tester("src/circuit_tests/storer-test.circom");

    const root = merkelize([aHash, bHash, cHash, dHash]);

    const parentHashL = digest([aHash, bHash], 2);
    const parentHashR = digest([cHash, dHash], 2);

    await cir.calculateWitness({
      "chunks": [[a], [b], [c], [d]],
      "siblings": [
        [bHash, parentHashR],
        [aHash, parentHashR],
        [dHash, parentHashL],
        [cHash, parentHashL]],
      "hashes": [aHash, bHash, cHash, dHash],
      "path": [0, 1, 2, 3],
      "root": root,
      "salt": saltHash,
    }, true);
  });

  it("Should verify chunk is not correct and part of dataset", async () => {
    const cir = await wasm_tester("src/circuit_tests/storer-test.circom");

    const root = merkelize([aHash, bHash, cHash, dHash]);

    const parentHashL = digest([aHash, bHash], 2);
    const parentHashR = digest([cHash, dHash], 2);

    const fn = async () => {
      return await cir.calculateWitness({
        "chunks": [
          [salt], // wrong chunk
          [b],
          [c],
          [d]],
        "siblings": [
          [bHash, parentHashR],
          [aHash, parentHashR],
          [dHash, parentHashL],
          [cHash, parentHashL]],
        "hashes": [saltHash, bHash, cHash, dHash],
        "path": [0, 1, 2, 3],
        "root": root,
        "salt": saltHash,
      }, true);
    }

    assert.isRejected(
      fn(), Error,
      /Error: Error: Assert Failed.\nError in template StorageProver_7 line: 75/);
  });

  function range(start, end) {
    return Array(end - start + 1).fill().map((_, idx) => start + idx)
  }

  it("Should test poseidon digest", async () => {
    const cir = await wasm_tester("src/circuit_tests/poseidon-digest-test.circom");
    let input = range(0, 255).map((c) => BigInt(c));
    await cir.calculateWitness({
      "block": input,
      "hash": digest(input, 16),
    });
  });

  // it("Should prove digest with zkey file", async () => {
  //   let input = range(0, 255).map((c) => BigInt(c));
  //   const {proof, publicSignals} = await snarkjs.groth16.fullProve(
  //     {
  //       "block": input,
  //       "hash": digest(input, 16),
  //     },
  //     "src/circuit_tests/artifacts/poseidon-digest-test_js/poseidon-digest-test.wasm",
  //     "circuit_0000.zkey");

  //   const vKey = JSON.parse(fs.readFileSync("verification_key.json"));
  //   const res = await snarkjs.groth16.verify(vKey, publicSignals, proof);
  //   assert(res);
  // });
});
