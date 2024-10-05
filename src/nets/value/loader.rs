use super::{ValueNetwork, L2_SIZE, OUTPUT_BUCKET_COUNT};

// todo: quantisation and transposing the l2 weights and biases 
pub const fn convert(net: ValueNetwork) -> ValueNetwork {
    let mut output_weights = [0.0; L2_SIZE * OUTPUT_BUCKET_COUNT];
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
        feature_weights: net.feature_weights,
        feature_biases: net.feature_biases,
        l2_weights: net.l2_weights,
        l2_biases: net.l2_biases,
        output_weights,
        output_biases: net.output_biases,
    }
}