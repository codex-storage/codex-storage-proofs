pragma circom 2.1.0;

include "../../circuits/storer.circom";

component main { public [root, salt] } = StorageProver(256, 4, 2, 16);
