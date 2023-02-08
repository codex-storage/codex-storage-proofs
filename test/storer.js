const chai = require("chai");
const path = require("path");
const crypto = require("crypto");
const F1Field = require("ffjavascript").F1Field;
const Scalar = require("ffjavascript").Scalar;
const mimc7 = require("circomlibjs").mimc7;
const mimcsponge = require("circomlibjs").mimcsponge;
const { MerkleTree } = require("merkletreejs");
const {c} = require("circom_tester");
const chaiAsPromised = require('chai-as-promised');

chai.use(chaiAsPromised);

exports.p = Scalar.fromString("21888242871839275222246405745257275088548364400416034343698204186575808495617");
const Fr = new F1Field(exports.p);

const assert = chai.assert;
const expect = chai.expect;

const wasm_tester = require("circom_tester").wasm;
const key = BigInt(2);

const digest = (buf) => mimc7.hash(buf, key);
const digestMulti = (buf) => mimc7.multiHash(buf, key);

function merkelize(leafs) {
  // simple merkle root (treehash) generator
  // unbalanced trees will have the last leaf duplicated
  var merkle = leafs;

  while (merkle.length > 1) {
    var newMerkle = [];

    var i = 0;
    while (i < merkle.length) {
      newMerkle.push(digestMulti([merkle[i], merkle[i + 1]]));
      i += 2;
    }

    if (merkle.length % 2 == 1) {
      newMerkle.add(digestMulti([merkle[merkle.length - 2], merkle[merkle.length - 2]]));
    }

    merkle = newMerkle;
  }

  return merkle[0];
}

describe("Storer test", function () {
  this.timeout(100000);

  const a = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const aHash = digestMulti(a);
  const b = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const bHash = digestMulti(b);
  const c = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const cHash = digestMulti(c);
  const d = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const dHash = digestMulti(d);
  const salt = Array.from(crypto.randomBytes(32).values()).map((v) => BigInt(v));
  const saltHash = digestMulti(salt);

  it("Should merkelize", async () => {
    let root = merkelize([aHash, bHash]);
    let hash = digestMulti([aHash, bHash]);

    assert.equal(hash, root);
  });

  it("Should verify chunk is correct and part of dataset", async () => {
    const cir = await wasm_tester(path.join(__dirname, "./circuits", "storer_test.circom"));

    const root = merkelize([aHash, bHash, cHash, dHash]);

    const parentHashL = digestMulti([aHash, bHash]);
    const parentHashR = digestMulti([cHash, dHash]);

    await cir.calculateWitness({
      "chunks": [[a], [b], [c], [d]],
      "siblings": [[bHash, parentHashR], [aHash, parentHashR], [dHash, parentHashL], [cHash, parentHashL]],
      "hashes": [aHash, bHash, cHash, dHash],
      "path": [0, 1, 2, 3],
      "root": root,
      "salt": saltHash,
    }, true);
  }).timeout(100000);

  it("Should verify chunk is correct and part of dataset", async () => {
    const cir = await wasm_tester(path.join(__dirname, "./circuits", "storer_test.circom"));

    const root = merkelize([aHash, bHash, cHash, dHash]);

    const parentHashL = digestMulti([aHash, bHash]);
    const parentHashR = digestMulti([cHash, dHash]);

    const fn = async () => {
      return await cir.calculateWitness({
        "chunks": [[salt], [b], [c], [d]],
        "siblings": [[bHash, parentHashR], [aHash, parentHashR], [dHash, parentHashL], [cHash, parentHashL]],
        "hashes": [saltHash, bHash, cHash, dHash],
        "path": [0, 1, 2, 3],
        "root": root,
        "salt": saltHash,
      }, true);
    }

    assert.isRejected(
      fn(), Error,
      /Error: Error: Assert Failed.\nError in template StorageProver_7 line: 75/);

  }).timeout(100000);
});
