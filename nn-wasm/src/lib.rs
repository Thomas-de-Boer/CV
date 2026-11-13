//! The browser's door into `nn-core`. Only flat arrays cross the boundary.

use nn_core::Network;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Model {
    network: Network,
}

#[wasm_bindgen]
impl Model {
    /// Loads a `.bin` weights file.
    pub fn from_bytes(bytes: &[u8]) -> Result<Model, JsError> {
        let network = Network::from_bytes(bytes).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Model { network })
    }

    /// `input`: 784 floats in 0.0..=1.0.
    /// Returns all activations concatenated: `[hidden1 | hidden2 | ... | output]`
    /// (empty if the input has the wrong length).
    pub fn predict(&self, input: &[f32]) -> Vec<f32> {
        if input.len() != self.network.input_size() {
            return Vec::new();
        }
        self.network.forward(input).concat()
    }

    /// Neuron counts including the input layer, e.g. `[784, 64, 32, 10]`.
    pub fn layer_sizes(&self) -> Vec<u32> {
        self.network.layer_sizes().iter().map(|s| *s as u32).collect()
    }

    /// Weights of one layer transition (0 = input → hidden 1), row-major:
    /// `outputs × inputs`. Empty if there is no such transition.
    pub fn weights(&self, layer: u32) -> Vec<f32> {
        self.network.layers().get(layer as usize).map(|l| l.weights.as_slice().to_vec()).unwrap_or_default()
    }

    /// Biases of one layer transition.
    pub fn biases(&self, layer: u32) -> Vec<f32> {
        self.network.layers().get(layer as usize).map(|l| l.biases.clone()).unwrap_or_default()
    }
}

/// Turns a drawing (`width × height` floats, ink = high) into the 28×28 image
/// the network sees: bounding box → 20×20 box → centred by centre of mass.
#[wasm_bindgen]
pub fn preprocess(pixels: &[f32], width: u32, height: u32) -> Vec<f32> {
    if pixels.len() != (width as usize) * (height as usize) {
        return vec![0.0; nn_core::preprocess::SIZE * nn_core::preprocess::SIZE];
    }
    nn_core::preprocess::preprocess(pixels, width as usize, height as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nn_core::{Activation, Layer, Matrix};

    fn model() -> Model {
        let hidden = Layer::new(Matrix::from_vec(2, 3, vec![1.0, 0.0, 0.0, 0.0, 1.0, 1.0]), vec![0.0, 0.5], Activation::Relu);
        let out = Layer::new(Matrix::from_vec(2, 2, vec![1.0, 0.0, 0.0, 1.0]), vec![0.0, 0.0], Activation::Softmax);
        Model { network: Network::new(vec![hidden, out]) }
    }

    #[test]
    fn predict_concatenates_all_layers() {
        let m = model();
        let acts = m.predict(&[1.0, 2.0, 3.0]);
        assert_eq!(acts.len(), 4);
        assert_eq!(&acts[..2], &[1.0, 5.5]);
        assert!((acts[2] + acts[3] - 1.0).abs() < 1e-6);
        assert!(m.predict(&[1.0]).is_empty());
    }

    #[test]
    fn sizes_and_weights_are_exposed() {
        let m = model();
        assert_eq!(m.layer_sizes(), vec![3, 2, 2]);
        assert_eq!(m.weights(0).len(), 6);
        assert_eq!(m.biases(0), vec![0.0, 0.5]);
        assert!(m.weights(5).is_empty());
    }

    #[test]
    fn preprocess_guards_against_bad_sizes() {
        assert_eq!(preprocess(&[1.0; 10], 5, 5).len(), 784);
        assert_eq!(preprocess(&[1.0; 25], 5, 5).len(), 784);
    }
}
