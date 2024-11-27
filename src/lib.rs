use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Statistics {
    pub length: usize, // 0 means all
    pub sentences: usize,
    pub characters: usize,
    pub distances: usize,
    pub successes: usize,
    pub translation_errors: usize,
    pub segmentation_errors: usize,
}
