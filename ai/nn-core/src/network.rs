//! Fully connected layers and the forward pass.

use crate::activation::Activation;
use crate::matrix::Matrix;

/// One transition between two layers of neurons.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// `outputs × inputs`: row `j` holds the incoming weights of neuron `j`.
    pub weights: Matrix,
    pub biases: Vec<f32>,
    pub activation: Activation,
}

impl Layer {
    pub fn new(weights: Matrix, biases: Vec<f32>, activation: Activation) -> Self {
        assert_eq!(weights.rows(), biases.len(), "one bias per output neuron");
        Layer { weights, biases, activation }
    }

    pub fn inputs(&self) -> usize {
        self.weights.cols()
    }

    pub fn outputs(&self) -> usize {
        self.weights.rows()
    }

    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut out = self.weights.mul_vec(input);
        for (o, b) in out.iter_mut().zip(&self.biases) {
            *o += b;
        }
        self.activation.apply(&mut out);
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Network {
    layers: Vec<Layer>,
}

impl Network {
    /// Panics if there are no layers or consecutive layers don't fit together.
    pub fn new(layers: Vec<Layer>) -> Self {
        assert!(!layers.is_empty(), "a network needs at least one layer");
        for pair in layers.windows(2) {
            assert_eq!(pair[0].outputs(), pair[1].inputs(), "layer sizes don't match");
        }
        Network { layers }
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// For training: the layer shapes must stay as they are.
    pub fn layers_mut(&mut self) -> &mut [Layer] {
        &mut self.layers
    }

    /// Neuron counts including the input, e.g. `[784, 64, 32, 10]`.
    pub fn layer_sizes(&self) -> Vec<usize> {
        let mut sizes = vec![self.layers[0].inputs()];
        sizes.extend(self.layers.iter().map(Layer::outputs));
        sizes
    }

    pub fn input_size(&self) -> usize {
        self.layers[0].inputs()
    }

    /// Runs the network and returns the activations of every layer after the
    /// input: `[hidden1, hidden2, ..., output]`.
    pub fn forward(&self, input: &[f32]) -> Vec<Vec<f32>> {
        assert_eq!(input.len(), self.input_size(), "wrong input size");
        let mut activations: Vec<Vec<f32>> = Vec::with_capacity(self.layers.len());
        for layer in &self.layers {
            let next = layer.forward(activations.last().map_or(input, Vec::as_slice));
            activations.push(next);
        }
        activations
    }

    /// Only the output layer.
    pub fn output(&self, input: &[f32]) -> Vec<f32> {
        self.forward(input).pop().expect("at least one layer")
    }

    /// Index of the strongest output neuron.
    pub fn classify(&self, input: &[f32]) -> usize {
        argmax(&self.output(input))
    }
}

pub fn argmax(values: &[f32]) -> usize {
    let mut best = 0;
    for (i, v) in values.iter().enumerate() {
        if *v > values[best] {
            best = i;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Network {
        // 2 inputs -> 2 hidden (ReLU) -> 2 outputs (softmax)
        let hidden = Layer::new(
            Matrix::from_vec(2, 2, vec![1.0, -1.0, 0.5, 0.5]),
            vec![0.0, -1.0],
            Activation::Relu,
        );
        let out = Layer::new(
            Matrix::from_vec(2, 2, vec![1.0, 0.0, 0.0, 1.0]),
            vec![0.0, 0.0],
            Activation::Softmax,
        );
        Network::new(vec![hidden, out])
    }

    #[test]
    fn forward_matches_hand_computation() {
        let acts = tiny().forward(&[3.0, 1.0]);
        // hidden: relu(3 - 1 + 0) = 2, relu(1.5 + 0.5 - 1) = 1
        assert_eq!(acts[0], vec![2.0, 1.0]);
        // softmax([2, 1]) = [e/(e+1), 1/(e+1)]
        let e = std::f32::consts::E;
        assert!((acts[1][0] - e / (e + 1.0)).abs() < 1e-6);
        assert!((acts[1][1] - 1.0 / (e + 1.0)).abs() < 1e-6);
    }

    #[test]
    fn relu_hides_negative_sums() {
        let acts = tiny().forward(&[0.0, 1.0]);
        assert_eq!(acts[0], vec![0.0, 0.0]);
        assert_eq!(acts[1], vec![0.5, 0.5]);
    }

    #[test]
    fn sizes_and_classify() {
        let n = tiny();
        assert_eq!(n.layer_sizes(), vec![2, 2, 2]);
        assert_eq!(n.classify(&[3.0, 1.0]), 0);
        assert_eq!(argmax(&[0.1, 0.7, 0.2]), 1);
    }

    #[test]
    #[should_panic]
    fn mismatched_layers_are_rejected() {
        let a = Layer::new(Matrix::zeros(3, 2), vec![0.0; 3], Activation::Relu);
        let b = Layer::new(Matrix::zeros(2, 4), vec![0.0; 2], Activation::Softmax);
        Network::new(vec![a, b]);
    }
}
