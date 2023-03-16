include "../node_modules/circomlib/circuits/poseidon.circom";

template SimpleHasher(SIZE) {
    signal input in[SIZE];
    signal input hash;

    component hasher = Poseidon(SIZE);
    hasher.inputs[0] <== in;
    hasher.out === hash;
}

component main = SimpleHasher(2);
