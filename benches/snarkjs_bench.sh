#!/bin/bash

# Set up the benchmarking parameters
ITERATIONS=10
CIRCUIT=../test/circuits/storer_test.circom
WITNESS=./input.json

# Define the SnarkJS commands for each system
GROTH16_CMD="snarkjs groth16 prove circuit_final.zkey witness.wtns proof.json public.json"
PLONK_CMD="snarkjs plonk prove circuit_final.zkey witness.wtns proof.json public.json"

# Set up the powers of tau ceremony
echo "Set up powers of tau ceremony"
snarkjs powersoftau new bn128 17 ../scripts/pot17_bn128_0000.ptau -v

# Generate circuit files
circom ${CIRCUIT} --r1cs --wasm --sym
snarkjs r1cs export json ./storer_test.r1cs ./storer_test.r1cs.json

# Generate the proving and verifying keys for Groth16
echo "Preparing phase 1"
snarkjs powersoftau contribute ../scripts/pot17_bn128_0000.ptau ../scripts/pot17_bn128_0001.ptau >/dev/null 2>&1 </dev/urandom
snarkjs powersoftau contribute ../scripts/pot17_bn128_0001.ptau ../scripts/pot17_bn128_0002.ptau >/dev/null 2>&1 </dev/urandom
snarkjs powersoftau verify ../scripts/pot17_bn128_0002.ptau
snarkjs powersoftau beacon ../scripts/pot17_bn128_0002.ptau ../scripts/pot17_bn128_beacon.ptau 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -n="Final Beacon"

echo "Preparing phase 2"
snarkjs powersoftau prepare phase2 ../scripts/pot17_bn128_beacon.ptau ../scripts/pot17_bn128_final.ptau
snarkjs powersoftau verify ../scripts/pot17_bn128_final.ptau

echo "Calculating witness"
node ./storer_test_js/generate_witness.js ./storer_test_js/storer_test.wasm ${WITNESS} ./witness.wtns
snarkjs wtns check ./storer_test.r1cs ./witness.wtns

# Benchmark Groth16
echo "Benchmarking Groth16..."
snarkjs groth16 setup ./storer_test.r1cs ../scripts/pot17_bn128_final.ptau circuit_0000.zkey
snarkjs zkey contribute circuit_0000.zkey circuit_0001.zkey --name="1st contributor" >/dev/null 2>&1
snarkjs zkey contribute circuit_0001.zkey circuit_0002.zkey --name="2nd contributor" >/dev/null 2>&1
snarkjs zkey verify ./storer_test.r1cs ../scripts/pot17_bn128_final.ptau circuit_0002.zkey
snarkjs zkey beacon circuit_0002.zkey circuit_final.zkey 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
snarkjs zkey verify ./storer_test.r1cs ../scripts/pot17_bn128_final.ptau circuit_final.zkey
snarkjs zkey export verificationkey circuit_final.zkey verification_key.json
for i in $(seq 1 $ITERATIONS); do
  echo "Proving..."
  /usr/bin/time -f "%e seconds" $GROTH16_CMD >/dev/null 2>&1
  echo "Verifying.."
  /usr/bin/time -f "%e seconds " snarkjs groth16 verify verification_key.json public.json proof.json
done


# Generate the proving and verifying keys for PLONK
echo "Generating PLONK keys..."
snarkjs powersoftau contribute ./contributions_2 pot12_0000_final_challenge >/dev/null 2>&1
snarkjs powersoftau verify ./contributions_2 >/dev/null 2>&1
snarkjs powersoftau prepare phase2 ./contributions_2 pot12_0000_final_challenge --srs_monomial_form ./srs.monomial >/dev/null 2>&1
snarkjs plonk setup --srs_monomial_form ./srs.monomial >/dev/null 2>&1


# Benchmark PLONK
echo "Benchmarking PLONK..."
for i in $(seq 1 $ITERATIONS); do
  /usr/bin/time -f "%e seconds" $PLONK_CMD >/dev/null 2>&1
done

