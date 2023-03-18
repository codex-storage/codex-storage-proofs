mod constants;

use ark_bn254::Fr;
use ark_ff::{Field, Zero};
use ruint::aliases::U256;

const N_ROUNDS_F: u8 = 8;
const N_ROUNDS_P: [i32; 16] = [
    56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
];

// Compute a Poseidon hash function of the input vector.
//
// # Panics
//
// Panics if `input` is not a valid field element.
#[must_use]
pub fn hash(inputs: &[U256]) -> U256 {
    assert!(inputs.len() > 0);
    assert!(inputs.len() <= N_ROUNDS_P.len());

    let t = inputs.len() + 1;
    let n_rounds_f = N_ROUNDS_F as usize;
    let n_rounds_p = N_ROUNDS_P[t - 2] as usize;
    let c = constants::C_CONST[t - 2].clone();
    let s = constants::S_CONST[t - 2].clone();
    let m = constants::M_CONST[t - 2].clone();
    let p = constants::P_CONST[t - 2].clone();

    let mut state: Vec<Fr> = inputs.iter().map(|f| f.try_into().unwrap()).collect();
    state.insert(0, Fr::zero());

    state = state.iter().enumerate().map(|(j, a)| *a + c[j]).collect();

    for r in 0..(n_rounds_f / 2 - 1) {
        state = state
            .iter()
            .map(|a| a.pow(&[5]))
            .enumerate()
            .map(|(i, a)| a + c[(r + 1) * t + i])
            .collect();

        state = state
            .iter()
            .enumerate()
            .map(|(i, _)| {
                state
                    .iter()
                    .enumerate()
                    .fold((0, Fr::zero()), |acc, item| {
                        (0, (acc.1 + m[item.0][i] * item.1))
                    })
                    .1
            })
            .collect();
    }

    state = state
        .iter()
        .map(|a| a.pow(&[5]))
        .enumerate()
        .map(|(i, a)| a + c[(n_rounds_f / 2 - 1 + 1) * t + i])
        .collect();

    state = state
        .iter()
        .enumerate()
        .map(|(i, _)| {
            state
                .iter()
                .enumerate()
                .fold((0, Fr::zero()), |acc, item| {
                    (0, (acc.1 + p[item.0][i] * item.1))
                })
                .1
        })
        .collect();

    for r in 0..n_rounds_p as usize {
        state[0] = state[0].pow(&[5]);
        state[0] = state[0] + c[(n_rounds_f / 2 + 1) * t + r];

        let s0 = state
            .iter()
            .enumerate()
            .fold((0, Fr::zero()), |acc, item| {
                (0, acc.1 + s[(t * 2 - 1) * r + item.0] * item.1)
            })
            .1;

        for k in 1..t {
            state[k] = state[k] + state[0] * s[(t * 2 - 1) * r + t + k - 1];
        }
        state[0] = s0;
    }

    for r in 0..(n_rounds_f / 2 - 1) as usize {
        state = state
            .iter()
            .map(|a| a.pow(&[5]))
            .enumerate()
            .map(|(i, a)| a + c[(n_rounds_f / 2 + 1) * t + n_rounds_p + r * t + i])
            .collect();

        state = state
            .iter()
            .enumerate()
            .map(|(i, _)| {
                state
                    .iter()
                    .enumerate()
                    .fold((0, Fr::zero()), |acc, item| {
                        (0, acc.1 + m[item.0][i] * item.1)
                    })
                    .1
            })
            .collect();
    }

    state = state.iter().map(|a| a.pow(&[5])).collect();
    state = state
        .iter()
        .enumerate()
        .map(
            |(i, _)| {
                state
                    .iter()
                    .enumerate()
                    .fold((0, Fr::zero()), |acc, item| {
                        (0, acc.1 + m[item.0][i] * item.1)
                    })
                    .1
            }, // reduce((acc, a, j) => F.add(acc, F.mul(M[j][i], a)), F.zero)
        )
        .collect();

    state[0].into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruint::uint;

    #[test]
    fn test_hash_inputs() {
        uint! {
            assert_eq!(hash(&[0_U256]), 0x2a09a9fd93c590c26b91effbb2499f07e8f7aa12e2b4940a3aed2411cb65e11c_U256);
            assert_eq!(hash(&[0_U256, 0_U256]), 0x2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864_U256);
            assert_eq!(hash(&[0_U256, 0_U256, 0_U256]), 0xbc188d27dcceadc1dcfb6af0a7af08fe2864eecec96c5ae7cee6db31ba599aa_U256);
            assert_eq!(hash(&[31213_U256, 132_U256]), 0x303f59cd0831b5633bcda50514521b33776b5d4280eb5868ba1dbbe2e4d76ab5_U256);
        }
    }
}
