use super::{PolicyNetwork, OW_SIZE, SUBNET_COUNT};

pub const QA: f32 = 512.0;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
#[repr(align(64))]
pub struct RawPolicyNetwork {
    pub output_weights: [f32; OW_SIZE],
    pub output_biases: [f32; SUBNET_COUNT],
}

// gonna quantise it
pub fn load(net: RawPolicyNetwork) -> Box<PolicyNetwork> {
    let mut output_weights = [0; OW_SIZE];
    let mut output_biases = [0; SUBNET_COUNT];
    for (index, weight) in net.output_weights.into_iter().enumerate() {
        output_weights[index] = (weight * QA) as i16;
    }
    for (index, bias) in net.output_biases.into_iter().enumerate() {
        output_biases[index] = (bias * QA) as i16;
    }
    Box::new(PolicyNetwork {
        output_weights,
        output_biases,
    })
}