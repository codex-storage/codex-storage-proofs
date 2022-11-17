pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/sha256/sha256.circom";
include "../node_modules/circomlib/circuits/poseidon.circom";

include "../node_modules/circomlib/circuits/bitify.circom";
include "tree.circom";

template HashCheck(blockSize) {
    signal input block[blockSize];
    //signal input blockHash[256];
    signal input blockHash;

    //component hash = Sha256(blockSize);
    component hash = Poseidon(blockSize);
    for (var i = 0; i < blockSize; i++) {
        hash.inputs[i] <== block[i];
    }
    hash.out === blockHash; //is this checking the whole array?
    // is this enough or do we need output?
}

template StorageProver(blockSize, qLen, nLevels) {
    // blockSize: size of block in bits (sha256), or in symbols (Poseidon)
    // qLen: query length, i.e. number if indices to be proven
    // nLevels: size of Merkle Tree in the manifest
    signal input chunks[qLen][blockSize];
    //signal input chunkHashes[qLen][256];
    signal input chunkHashes[qLen];
    signal input indices[qLen];
    signal input treeSiblings[qLen][nLevels];

    signal input root;

    //check that chunks hash to given hashes
    component hashCheck[qLen];
    for (var i = 0; i < qLen; i++) {
        hashCheck[i] = HashCheck(blockSize);
        hashCheck[i].block <== chunks[i];
        hashCheck[i].blockHash <== chunkHashes[i];
    }

    //check that the tree is correct
    // - check indices against limits
    // - convert indices to treePathIndices
    // - check chunkHash and treeSiblings according to treePathIndices against root

    component num2Bits[qLen];
    component inclusionProofs[qLen];
    for (var i = 0; i < qLen; i++) {

        num2Bits[i] = Num2Bits(nLevels);
        num2Bits[i].in <== indices[i];

        inclusionProofs[i] = MerkleTreeInclusionProof(nLevels);
        inclusionProofs[i].leaf <== chunkHashes[i];
        for (var j = 0; j < nLevels; j++) {
            inclusionProofs[i].siblings[j] <== treeSiblings[i][j];
            inclusionProofs[i].pathIndices[j] <== num2Bits[i].out[j];
        }
        root === inclusionProofs[i].root;
    }
}

//component main {public [blockHash]} = HashCheck(512);
//component main {public [indices]} = StorageProver(512, 1, 10);
component main {public [indices]} = StorageProver(2, 10, 10);
