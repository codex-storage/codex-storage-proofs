include "../node_modules/circomlib/circuits/poseidon.circom";

template SimpleHasher(SIZE) {
    signal input in[SIZE];
    signal input hash;

    component hasher = Poseidon(SIZE);
    for(var i = 0; i < SIZE; i++) {
        hasher.inputs[i] <== in[i];
    }

    hasher.out === hash;
}

component main = SimpleHasher(1);
