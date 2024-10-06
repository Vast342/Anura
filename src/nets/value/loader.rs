use super::{
    ValueNetwork, INPUT_BUCKET_COUNT, INPUT_SIZE, L1_SIZE, L2_SIZE, OUTPUT_BUCKET_COUNT, QA ,
};

pub struct UnquantisedValueNetwork {
    feature_weights: [f32; INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT],
    feature_biases: [f32; L1_SIZE],
    l2_weights: [[[f32; L2_SIZE]; OUTPUT_BUCKET_COUNT]; L1_SIZE],
    l2_biases: [[f32; L2_SIZE]; OUTPUT_BUCKET_COUNT],
    output_weights: [f32; L2_SIZE * OUTPUT_BUCKET_COUNT],
    output_biases: [f32; OUTPUT_BUCKET_COUNT],
}

pub const VALUE_NET_RAW: UnquantisedValueNetwork =
    unsafe { std::mem::transmute(*include_bytes!("avn_007.vn")) };

fn quantise(value: f32, constant: usize) -> i32 {
    (value * constant as f32).round() as i32
}

// quantises the net and transposes output weights
pub fn convert(net: UnquantisedValueNetwork) -> Box<ValueNetwork> {
    let mut feature_weights = [0; INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT];

    for feature in 0..INPUT_SIZE * L1_SIZE * INPUT_BUCKET_COUNT {
        feature_weights[feature] = quantise(net.feature_weights[feature], QA);
    }

    let mut feature_biases = [0; L1_SIZE];

    for l1_node in 0..L1_SIZE {
        feature_biases[l1_node] = quantise(net.feature_biases[l1_node], QA)
    }

    let mut output_weights = [0.0; L2_SIZE * OUTPUT_BUCKET_COUNT];

    // output weights, quantised and then transposed
    for l2_node in 0..L2_SIZE {
        for bucket in 0..OUTPUT_BUCKET_COUNT {
            let src = l2_node * OUTPUT_BUCKET_COUNT + bucket;
            let dst = bucket * L2_SIZE + l2_node;
            output_weights[dst] = net.output_weights[src];
        }
    }

    Box::new(ValueNetwork {
        feature_weights,
        feature_biases,
        l2_weights: net.l2_weights,
        l2_biases: net.l2_biases,
        output_weights,
        output_biases: net.output_biases,
    })
}
