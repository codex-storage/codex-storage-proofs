const chai = require("chai");
const path = require("path");
const crypto = require("crypto");
const F1Field = require("ffjavascript").F1Field;
const Scalar = require("ffjavascript").Scalar;
exports.p = Scalar.fromString("21888242871839275222246405745257275088548364400416034343698204186575808495617");
const Fr = new F1Field(exports.p);

const assert = chai.assert;

const sha256 = require("./helpers/sha256");

const wasm_tester = require("circom_tester").wasm;

const snarkjs = require("snarkjs");
const buildBn128 = require("ffjavascript").buildBn128;

// const printSignal = require("./helpers/printsignal");


function buffer2bitArray(b) {
    const res = [];
    for (let i=0; i<b.length; i++) {
        for (let j=0; j<8; j++) {
            res.push((b[i] >> (7-j) &1));
        }
    }
    return res;
}

function bitArray2buffer(a) {
    const len = Math.floor((a.length -1 )/8)+1;
    const b = new Buffer.alloc(len);

    for (let i=0; i<a.length; i++) {
        const p = Math.floor(i/8);
        b[p] = b[p] | (Number(a[i]) << ( 7 - (i%8)  ));
    }
    return b;
}


describe("SHA256 test", function () {
    this.timeout(1000000);

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

    var cir;

    before( async () => {
        curve = await buildBn128();
//        curve.Fr.s = 10;
    });
    after( async () => {
        await curve.terminate();
        // console.log(process._getActiveHandles());
        // console.log(process._getActiveRequests());
    });

    it ("powersoftau new", async () => {
        await snarkjs.powersOfTau.newAccumulator(curve, 11, ptau_0);
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

    it("Should work bits to array and array to bits", async () => {
        const b = new Buffer.alloc(64);
        for (let i=0; i<64; i++) {
            b[i] = i+1;
        }
        const a = buffer2bitArray(b);
        const b2 = bitArray2buffer(a);

        assert.equal(b.toString("hex"), b2.toString("hex"), true);
    });

    it ("compile circuit 64 bytes (512 bits)", async () => {
        cir = await wasm_tester(path.join(__dirname, "../circuits", "sha256_test512.circom"));
    });

    it("Should calculate a hash of 2 compressor", async () => {

        const b = new Buffer.alloc(64);
        for (let i=0; i<64; i++) {
            b[i] = i+1;
        }

        const hash = crypto.createHash("sha256")
            .update(b)
            .digest("hex");

        const arrIn = buffer2bitArray(b);
        const witness = await cir.calculateWitness({ "in": arrIn }, true);
        console.warn("witness: ", witness.length, " bits");

        const arrOut = witness.slice(1, 257);
        const hash2 = bitArray2buffer(arrOut).toString("hex");

        assert.equal(hash, hash2);

    }).timeout(1000000);

    it ("compile circuit 640 bytes", async () => {
        cir = await wasm_tester(path.join(__dirname, "circuits", "sha256_test5120.circom"));
    });

    it("Should calculate a hash of 20 compressor", async () => {

        const b = new Buffer.alloc(640);
        for (let i=0; i<640; i++) {
            b[i] = i+1;
        }

        const hash = crypto.createHash("sha256")
            .update(b)
            .digest("hex");

        const arrIn = buffer2bitArray(b);
        const witness = await cir.calculateWitness({ "in": arrIn }, true);
        console.warn("witness: ", witness.length, " bits");

        const arrOut = witness.slice(1, 257);
        const hash2 = bitArray2buffer(arrOut).toString("hex");

        assert.equal(hash, hash2);

    }).timeout(1000000);

});
