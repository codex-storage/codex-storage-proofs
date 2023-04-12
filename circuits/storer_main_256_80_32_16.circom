pragma circom 2.1.0;

include "./storer.circom";

component main { public [root, salt] } = StorageProver(256, 80, 32, 16);
