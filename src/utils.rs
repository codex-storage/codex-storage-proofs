use crate::poseidon::hash1;
use ruint::aliases::U256;

fn digest(input: &[U256], chunk_size: Option<usize>) -> U256 {
    let chunk_size = chunk_size.unwrap_or(4);
    let chunks = ((input.len() as f32) / (chunk_size as f32)).ceil() as usize;
    let mut concat = vec![];

    let mut i: usize = 0;
    while i < chunks {
        let range = (i * chunk_size)..std::cmp::min((i + 1) * chunk_size, input.len());
        let mut chunk: Vec<Fq> = input[range].to_vec();
        if chunk.len() < chunk_size {
            chunk.resize(chunk_size as usize, Fq::zero());
        }

        concat.push(hash1(chunk)?);
        i += chunk_size;
    }

    if concat.len() > 1 {
        return hasher(concat);
    }

    return Ok(concat[0]);
}
