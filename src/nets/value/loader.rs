use super::{ValueNetwork, INPUT_BUCKET_COUNT, INPUT_BUCKET_SCHEME, INPUT_SIZE, L1_SIZE, L2_SIZE, OUTPUT_BUCKET_COUNT};

pub struct UnquantisedValueNetwork {
    feature_weights: [f32; INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT],
    feature_biases: [f32; L1_SIZE],
    l2_weights: [f32; L2_SIZE * L1_SIZE * OUTPUT_BUCKET_COUNT],
    l2_biases: [f32; L2_SIZE * OUTPUT_BUCKET_COUNT],
    output_weights: [f32; L1_SIZE * OUTPUT_BUCKET_COUNT],
    output_biases: [f32; OUTPUT_BUCKET_COUNT],
}

pub const fn convert(net: UnquantisedValueNetwork) -> ValueNetwork {
    let mut feature_weights = [0; INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT];
    let mut feature_biases = [0; L1_SIZE];
    let mut l2_weights = [0; L2_SIZE * L1_SIZE * OUTPUT_BUCKET_COUNT];
    let mut l2_biases = [0; L2_SIZE * OUTPUT_BUCKET_COUNT];
    let mut output_weights = [0; L1_SIZE * OUTPUT_BUCKET_COUNT];
    let mut output_biases = [0; OUTPUT_BUCKET_COUNT];

    let mut weight = 0;
    // output weights
    while weight < L1_SIZE {
        let mut bucket = 0;
        while bucket < OUTPUT_BUCKET_COUNT {
            let src = weight * OUTPUT_BUCKET_COUNT + bucket;
            let dst = bucket * L1_SIZE + weight;
            output_weights[dst] = net.output_weights[src];
            bucket += 1;
        }
        weight += 1;
    }
    ValueNetwork {
        feature_weights,
        feature_biases,
        l2_weights,
        l2_biases,
        output_weights,
        output_biases,
    }
}