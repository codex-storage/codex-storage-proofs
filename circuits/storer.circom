pragma circom 2.1.0;

include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/switcher.circom";
include "../node_modules/circomlib/circuits/bitify.circom";

template parallel MerkleProof(LEVELS) {
    signal input leaf;
    signal input pathElements[LEVELS];
    signal input pathIndices;

    signal output root;

    component switcher[LEVELS];
    component hasher[LEVELS];

    component indexBits = Num2Bits(LEVELS);
    indexBits.in <== pathIndices;

    for (var i = 0; i < LEVELS; i++) {
        switcher[i] = Switcher();

        switcher[i].L <== i == 0 ? leaf : hasher[i - 1].out;
        switcher[i].R <== pathElements[i];
        switcher[i].sel <== indexBits.out[i];

        hasher[i] = Poseidon(2);
        hasher[i].inputs[0] <== switcher[i].outL;
        hasher[i].inputs[1] <== switcher[i].outR;
    }

    root <== hasher[LEVELS - 1].out;
}

function roundUpDiv(x, n) {
    var last = x % n; // get the last digit
    var div = x \ n; // get the division

    if (last > 0) {
        return div + 1;
    } else {
        return div;
    }
}

template parallel HashCheck(BLOCK_SIZE, CHUNK_SIZE) {
    signal input block[BLOCK_SIZE];
    signal output hash;

    // Split array into chunks of size CHUNK_SIZE
    var NUM_CHUNKS = roundUpDiv(BLOCK_SIZE, CHUNK_SIZE);

    // Initialize an array to store hashes of each block
    component hashes[NUM_CHUNKS];

    // Loop over chunks and hash them using Poseidon()
    for (var i = 0; i < NUM_CHUNKS; i++) {
        hashes[i] = Poseidon(CHUNK_SIZE);

        var start = i * CHUNK_SIZE;
        var end = start + CHUNK_SIZE;
        for (var j = start; j < end; j++) {
            if (j >= BLOCK_SIZE) {
                hashes[i].inputs[j - start] <== 0;
            } else {
                hashes[i].inputs[j - start] <== block[j];
            }
        }
    }

    // Concatenate hashes into a single block
    var concat[NUM_CHUNKS];
    for (var i = 0; i < NUM_CHUNKS; i++) {
        concat[i] = hashes[i].out;
    }

    // Hash concatenated array using Poseidon() again
    component h = Poseidon(NUM_CHUNKS);
    h.inputs <== concat;

    // Assign output to hash signal
    hash <== h.out;
}

template StorageProver(BLOCK_SIZE, QUERY_LEN, LEVELS, CHUNK_SIZE) {
    // BLOCK_SIZE: size of block in symbols
    // QUERY_LEN: query length, i.e. number if indices to be proven
    // LEVELS: size of Merkle Tree in the manifest
    // CHUNK_SIZE: number of symbols to hash in one go
    signal input chunks[QUERY_LEN][BLOCK_SIZE]; // chunks to be proven
    signal input siblings[QUERY_LEN][LEVELS];   // siblings hashes of chunks to be proven
    signal input path[QUERY_LEN];               // path of chunks to be proven
    signal input hashes[QUERY_LEN];             // hashes of chunks to be proven
    signal input root;                          // root of the Merkle Tree
    signal input salt;                          // salt (block hash) to prevent preimage attacks

    signal saltSquare <== salt * salt;          // might not be necesary as it's part of the public inputs

    component hashers[QUERY_LEN];
    for (var i = 0; i < QUERY_LEN; i++) {
        hashers[i] = HashCheck(BLOCK_SIZE, CHUNK_SIZE);
        hashers[i].block <== chunks[i];
        hashers[i].hash === hashes[i];
    }

    component merkelizer[QUERY_LEN];
    for (var i = 0; i < QUERY_LEN; i++) {
        merkelizer[i] = MerkleProof(LEVELS);
        merkelizer[i].leaf <== hashes[i];
        merkelizer[i].pathElements <== siblings[i];
        merkelizer[i].pathIndices <== path[i];

        merkelizer[i].root === root;
    }
}
