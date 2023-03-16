pragma circom 2.1.0;

include "../../circuits/storer.circom";

// component main { public [root, salt] } = StorageProver(32, 4, 2, 4);
component main { public [root, salt] } = StorageProver(32, 4, 2, 2);
