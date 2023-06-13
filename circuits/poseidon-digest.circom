pragma circom 2.1.0;

include "../node_modules/circomlib/circuits/poseidon.circom";

function roundUpDiv(x, n) {
    var last = x % n; // get the last digit
    var div = x \ n; // get the division

    if (last > 0) {
        return div + 1;
    }

    return div;
}

template parallel PoseidonDigest(BLOCK_SIZE, DIGEST_CHUNK) {
    // BLOCK_SIZE - size of the input block array
    // DIGEST_CHUNK - number of elements to hash at once
    signal input block[BLOCK_SIZE]; // Input block array
    signal output hash; // Output hash

    // Split array into chunks of size DIGEST_CHUNK, usually 2
    var NUM_CHUNKS = roundUpDiv(BLOCK_SIZE, DIGEST_CHUNK);

    // Initialize an array to store hashes of each block
    component hashes[NUM_CHUNKS];

    // Loop over chunks and hash them using Poseidon()
    for (var i = 0; i < NUM_CHUNKS; i++) {
        hashes[i] = Poseidon(DIGEST_CHUNK);

        var start = i * DIGEST_CHUNK;
        var end = start + DIGEST_CHUNK;
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
