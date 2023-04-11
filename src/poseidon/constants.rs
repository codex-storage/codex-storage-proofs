use ark_bn254::Fr;
use num_bigint::BigUint;
use once_cell::sync::Lazy;
use num_traits::Num;

const CONSTANTS_STR: &str = include_str!("poseidon_constants_opt.json");

pub static CONSTANTS: Lazy<serde_json::Value> = Lazy::new(|| {
    serde_json::from_str(CONSTANTS_STR).unwrap()
});

pub static C_CONST: Lazy<Vec<Vec<Fr>>> = Lazy::new(|| {
    CONSTANTS["C"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    Fr::try_from(
                        BigUint::from_str_radix(
                            c.as_str().unwrap().strip_prefix("0x").unwrap(),
                            16,
                        )
                        .unwrap(),
                    )
                })
                .collect::<Result<Vec<Fr>, _>>()
                .unwrap()
        })
        .collect::<Vec<Vec<Fr>>>()
});

pub static S_CONST: Lazy<Vec<Vec<Fr>>> = Lazy::new(|| {
    CONSTANTS["S"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    Fr::try_from(
                        BigUint::from_str_radix(
                            c.as_str().unwrap().strip_prefix("0x").unwrap(),
                            16,
                        )
                        .unwrap(),
                    )
                })
                .collect::<Result<Vec<Fr>, _>>()
                .unwrap()
        })
        .collect::<Vec<Vec<Fr>>>()
});

pub static M_CONST: Lazy<Vec<Vec<Vec<Fr>>>> = Lazy::new(|| {
    CONSTANTS["M"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    c.as_array()
                        .unwrap()
                        .iter()
                        .map(|c| {
                            Fr::try_from(
                                BigUint::from_str_radix(
                                    c.as_str().unwrap().strip_prefix("0x").unwrap(),
                                    16,
                                )
                                .unwrap(),
                            )
                        })
                        .collect::<Result<Vec<Fr>, _>>()
                        .unwrap()
                })
                .collect()
        })
        .collect::<Vec<Vec<Vec<Fr>>>>()
});

pub static P_CONST: Lazy<Vec<Vec<Vec<Fr>>>> = Lazy::new(|| {
    CONSTANTS["P"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    c.as_array()
                        .unwrap()
                        .iter()
                        .map(|c| {
                            Fr::try_from(
                                BigUint::from_str_radix(
                                    c.as_str().unwrap().strip_prefix("0x").unwrap(),
                                    16,
                                )
                                .unwrap(),
                            )
                        })
                        .collect::<Result<Vec<Fr>, _>>()
                        .unwrap()
                })
                .collect()
        })
        .collect::<Vec<Vec<Vec<Fr>>>>()
});
