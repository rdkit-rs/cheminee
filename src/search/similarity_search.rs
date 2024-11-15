use bitvec::prelude::BitVec;
use cheminee_similarity_model::encoder::{build_encoder_model, EncoderModel};

lazy_static::lazy_static! {
    pub static ref ENCODER_MODEL: EncoderModel = build_encoder_model().unwrap();
}

pub fn encode_fingerprint(fingerprint: &[u8], only_best_cluster: bool) -> eyre::Result<Vec<i32>> {
    let bit_vec = BitVec::<u8>::from_slice(fingerprint);
    let fp_vec = bit_vec
        .iter()
        .map(|b| if *b { 1 } else { 0 })
        .collect::<Vec<u8>>();

    let ranked_clusters = ENCODER_MODEL.transform(&fp_vec)?;

    if only_best_cluster {
        Ok(vec![ranked_clusters[0]])
    } else {
        Ok(ranked_clusters)
    }
}
