#![allow(dead_code)]

use crate::poseidon::hash;
use ruint::{aliases::U256, uint};

pub fn digest(input: &[U256], chunk_size: Option<usize>) -> U256 {
    let chunk_size = chunk_size.unwrap_or(4);
    let chunks = ((input.len() as f32) / (chunk_size as f32)).ceil() as usize;
    let mut concat: Vec<U256> = vec![];

    for i in 0..chunks {
        let range = (i * chunk_size)..std::cmp::min((i + 1) * chunk_size, input.len());
        let mut chunk = input[range].to_vec();
        if chunk.len() < chunk_size {
            chunk.resize(chunk_size, uint!(0_U256));
        }

        concat.push(hash(chunk.as_slice()));
    }

    if concat.len() > 1 {
        return hash(concat.as_slice());
    }

    concat[0]
}

pub fn merkelize(leafs: &[U256]) -> U256 {
    // simple merkle root (treehash) generator
    // unbalanced trees will have the last leaf duplicated
    let mut merkle: Vec<U256> = leafs.to_vec();

    while merkle.len() > 1 {
        let mut new_merkle = Vec::new();
        let mut i = 0;
        while i < merkle.len() {
            new_merkle.push(hash(&[merkle[i], merkle[i + 1]]));
            i += 2;
        }

        if merkle.len() % 2 == 1 {
            new_merkle.push(hash(&[merkle[merkle.len() - 2], merkle[merkle.len() - 2]]));
        }

        merkle = new_merkle;
    }

    merkle[0]
}
