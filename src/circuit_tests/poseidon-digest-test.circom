pragma circom 2.1.0;

include "../../circuits/poseidon-digest.circom";

component main = PoseidonDigest(256, 16);
