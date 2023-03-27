#!/bin/bash

# Set up the benchmarking parameters
ITERATIONS=10
CIRCUIT=./my_circuit.circom
WITNESS=./my_witness.json

# Define the SnarkJS commands for each system
GROTH16_CMD="snarkjs groth16 fullprove -c ${CIRCUIT} -j ${WITNESS} -p ${PROVKEY}"
PLONK_CMD="snarkjs plonk prove -w ${WITNESS} -c ${CIRCUIT} -p ${PROVKEY} -v ${PUBKEY}"
POWERSOFTAU_CMD="snarkjs powersoftau new bn128 12"

# Prepare Powers of Tau contribution 1
echo "Preparing Powers of Tau contribution 1..."
snarkjs powersoftau prepare phase1 ${CIRCUIT} ${POWERSOFTAU_CMD} --name="Contribution 1" >/dev/null 2>&1

# Generate circuit files
circom ${CIRCUIT} --r1cs --wasm --sym

# Generate the proving and verifying keys for Groth16
echo "Generating Groth16 keys..."
snarkjs powersoftau contribute ./contributions_1 pot12_0000_final_challenge >/dev/null 2>&1
snarkjs powersoftau verify ./contributions_1 >/dev/null 2>&1
snarkjs powersoftau prepare phase2 ./contributions_1 pot12_0000_final_challenge --srs_monomial_form ./srs.monomial >/dev/null 2>&1
snarkjs groth16 setup --power 12 --srs_monomial_form ./srs.monomial >/dev/null 2>&1
PROVKEY=./proving_key.json
PUBKEY=./verification_key.json

# Benchmark Groth16
echo "Benchmarking Groth16..."
for i in $(seq 1 $ITERATIONS); do
  /usr/bin/time -f "%e seconds" $GROTH16_CMD >/dev/null 2>&1
done

# Prepare Powers of Tau contribution 2
echo "Preparing Powers of Tau contribution 2..."
snarkjs powersoftau prepare phase1 ${CIRCUIT} ${POWERSOFTAU_CMD} --name="Contribution 2" >/dev/null 2>&1

# Generate the proving and verifying keys for PLONK
echo "Generating PLONK keys..."
snarkjs powersoftau contribute ./contributions_2 pot12_0000_final_challenge >/dev/null 2>&1
snarkjs powersoftau verify ./contributions_2 >/dev/null 2>&1
snarkjs powersoftau prepare phase2 ./contributions_2 pot12_0000_final_challenge --srs_monomial_form ./srs.monomial >/dev/null 2>&1
snarkjs plonk setup --srs_monomial_form ./srs.monomial >/dev/null 2>&1
PROVKEY=./proving_key.bin
PUBKEY=./verification_key.json

# Benchmark PLONK
echo "Benchmarking PLONK..."
for i in $(seq 1 $ITERATIONS); do
  /usr/bin/time -f "%e seconds" $PLONK_CMD >/dev/null 2>&1
done

