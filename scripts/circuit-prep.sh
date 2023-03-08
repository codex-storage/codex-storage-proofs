#!/bin/bash
set -e
set -x

CIRCUIT=`basename $1`
POWER="${2:-12}"
CURVE="${3:-bn128}"

POTPREFIX=pot${POWER}_${CURVE}

if [ ! -f ${POTPREFIX}_final.ptau ]
then
    snarkjs powersoftau new $CURVE $POWER ${POTPREFIX}_0000.ptau -v
    snarkjs powersoftau contribute ${POTPREFIX}_0000.ptau ${POTPREFIX}_0001.ptau --name="First contribution" -v -e="random text"
    snarkjs powersoftau verify ${POTPREFIX}_0001.ptau
    snarkjs powersoftau beacon ${POTPREFIX}_0001.ptau ${POTPREFIX}_beacon.ptau 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -n="Final Beacon"
    snarkjs powersoftau prepare phase2 ${POTPREFIX}_beacon.ptau ${POTPREFIX}_final.ptau -v
    snarkjs powersoftau verify ${POTPREFIX}_final.ptau
fi

# phase 2
circom $1.circom --r1cs --wasm

snarkjs groth16 setup ${CIRCUIT}.r1cs ${POTPREFIX}_final.ptau ${CIRCUIT}_0000.zkey
snarkjs zkey contribute ${CIRCUIT}_0000.zkey ${CIRCUIT}_0001.zkey --name="1st Contributor Name" -v -e="another random text"
snarkjs zkey verify ${CIRCUIT}.r1cs ${POTPREFIX}_final.ptau ${CIRCUIT}_0001.zkey
snarkjs zkey beacon ${CIRCUIT}_0001.zkey ${CIRCUIT}_final.zkey 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -n="Final Beacon phase2"

