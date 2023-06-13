pragma circom 2.1.0;

include "../../circuits/poseidon-digest.circom";

template PoseidonDigestTest(BLOCK_SIZE, CHUNK_SIZE) {
    signal input block[BLOCK_SIZE];
    signal input hash;
    signal output hash2;

    component digest = PoseidonDigest(BLOCK_SIZE, CHUNK_SIZE);
    for (var i = 0; i < BLOCK_SIZE; i++) {
        digest.block[i] <== block[i];
    }

    digest.hash === hash; // verify that the hash is correct

    hash2 <== digest.hash;
}

component main { public [hash] } = PoseidonDigestTest(256, 16);
