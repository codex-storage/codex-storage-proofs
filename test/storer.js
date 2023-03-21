const chai = require("chai");
const path = require("path");
const crypto = require("crypto");
const F1Field = require("ffjavascript").F1Field;
const Scalar = require("ffjavascript").Scalar;
const {c} = require("circom_tester");
const chaiAsPromised = require('chai-as-promised');
const poseidon = require("circomlibjs/src/poseidon");
const wasm_tester = require("circom_tester").wasm;

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

  const a = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const aHash = digest(a);
  const b = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const bHash = digest(b);
  const c = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const cHash = digest(c);
  const d = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const dHash = digest(d);
  const salt = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const saltHash = digest(salt);

  it("Should merkelize", async () => {
    let root = merkelize([aHash, bHash]);
    let hash = digest([aHash, bHash], 2);

    assert.equal(hash, root);
  });

  it("Should verify chunk is correct and part of dataset", async () => {
    const cir = await wasm_tester(path.join(__dirname, "./circuits", "storer_test.circom"));

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
    const cir = await wasm_tester(path.join(__dirname, "./circuits", "storer_test.circom"));

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


  it("Should should hash item", async () => {
    console.log(digest([0, 0, 0]).toString(16));
  });
});
