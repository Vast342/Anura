use super::{ValueNetwork, INPUT_BUCKET_COUNT, INPUT_BUCKET_SCHEME, INPUT_SIZE, L1_SIZE, L2_SIZE, OUTPUT_BUCKET_COUNT, QA};

pub struct UnquantisedValueNetwork {
    feature_weights: [f32; INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT],
    feature_biases: [f32; L1_SIZE],
    l2_weights: [[[f32; L2_SIZE]; L1_SIZE]; OUTPUT_BUCKET_COUNT],
    l2_biases: [[f32; L2_SIZE]; OUTPUT_BUCKET_COUNT],
    output_weights: [f32; L2_SIZE * OUTPUT_BUCKET_COUNT],
    output_biases: [f32; OUTPUT_BUCKET_COUNT],
}

// todo: quantisation and transposing the l2 weights and biases 
pub const fn convert(net: UnquantisedValueNetwork) -> ValueNetwork {
    let mut feature_weights = [0; INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT];
    let mut feature_biases = [0; L1_SIZE];
    let mut output_weights = [0.0; L2_SIZE * OUTPUT_BUCKET_COUNT];
    let mut index = 0;
    while index < INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT {
        feature_weights[index] = net.feature_weights[index] as i32;
        index += 1;
    }
    index = 0;
    while index < L1_SIZE {
        feature_biases[index] = (net.feature_biases[index] * QA as f32) as i32;
        index += 1;
    }
    let mut weight = 0;
    // output weights
    while weight < L2_SIZE {
        let mut bucket = 0;
        while bucket < OUTPUT_BUCKET_COUNT {
            let src = weight * OUTPUT_BUCKET_COUNT + bucket;
            let dst = bucket * L2_SIZE + weight;
            output_weights[dst] = net.output_weights[src];
            bucket += 1;
        }
        weight += 1;
    }
    ValueNetwork {
        feature_weights,
        feature_biases,
        l2_weights: net.l2_weights,
        l2_biases: net.l2_biases,
        output_weights,
        output_biases: net.output_biases,
    }
}