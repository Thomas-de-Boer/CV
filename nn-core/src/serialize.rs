//! The weights file format (`.bin`), little-endian:
//!
//! ```text
//! magic    "DNN1"            4 bytes
//! n_sizes  u32               number of layer sizes (e.g. 4)
//! sizes    u32 × n_sizes     e.g. 784, 64, 32, 10
//! activ    u8 × (n_sizes-1)  0 = ReLU, 1 = sigmoid, 2 = softmax
//! then per transition: weights (rows × cols f32), biases (rows f32)
//! ```

use crate::activation::Activation;
use crate::matrix::Matrix;
use crate::network::{Layer, Network};
use std::fmt;

pub const MAGIC: &[u8; 4] = b"DNN1";

/// Refuse absurd sizes before allocating anything for them.
const MAX_LAYER_SIZE: u32 = 1 << 20;
const MAX_LAYERS: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    BadMagic,
    UnexpectedEof,
    BadLayerSizes,
    UnknownActivation(u8),
    TrailingBytes,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::BadMagic => write!(f, "not a DNN1 weights file"),
            FormatError::UnexpectedEof => write!(f, "weights file is truncated"),
            FormatError::BadLayerSizes => write!(f, "weights file has invalid layer sizes"),
            FormatError::UnknownActivation(c) => write!(f, "unknown activation code {c}"),
            FormatError::TrailingBytes => write!(f, "weights file has extra bytes at the end"),
        }
    }
}

impl std::error::Error for FormatError {}

impl Network {
    pub fn to_bytes(&self) -> Vec<u8> {
        let sizes = self.layer_sizes();
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(sizes.len() as u32).to_le_bytes());
        for s in &sizes {
            out.extend_from_slice(&(*s as u32).to_le_bytes());
        }
        for layer in self.layers() {
            out.push(layer.activation.code());
        }
        for layer in self.layers() {
            for v in layer.weights.as_slice().iter().chain(&layer.biases) {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Network, FormatError> {
        let mut r = Reader { bytes, pos: 0 };
        if r.take(4)? != MAGIC {
            return Err(FormatError::BadMagic);
        }
        let n_sizes = r.u32()?;
        if !(2..=MAX_LAYERS).contains(&n_sizes) {
            return Err(FormatError::BadLayerSizes);
        }
        let mut sizes = Vec::with_capacity(n_sizes as usize);
        for _ in 0..n_sizes {
            let s = r.u32()?;
            if s == 0 || s > MAX_LAYER_SIZE {
                return Err(FormatError::BadLayerSizes);
            }
            sizes.push(s as usize);
        }
        let mut activations = Vec::with_capacity(sizes.len() - 1);
        for _ in 1..sizes.len() {
            let code = r.take(1)?[0];
            activations.push(Activation::from_code(code).ok_or(FormatError::UnknownActivation(code))?);
        }
        let mut layers = Vec::with_capacity(activations.len());
        for (pair, activation) in sizes.windows(2).zip(activations) {
            let (cols, rows) = (pair[0], pair[1]);
            let weights = Matrix::from_vec(rows, cols, r.f32s(rows * cols)?);
            let biases = r.f32s(rows)?;
            layers.push(Layer::new(weights, biases, activation));
        }
        if r.pos != bytes.len() {
            return Err(FormatError::TrailingBytes);
        }
        Ok(Network::new(layers))
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], FormatError> {
        let end = self.pos.checked_add(n).ok_or(FormatError::UnexpectedEof)?;
        let slice = self.bytes.get(self.pos..end).ok_or(FormatError::UnexpectedEof)?;
        self.pos = end;
        Ok(slice)
    }

    fn u32(&mut self) -> Result<u32, FormatError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn f32s(&mut self, n: usize) -> Result<Vec<f32>, FormatError> {
        let len = n.checked_mul(4).ok_or(FormatError::UnexpectedEof)?;
        let b = self.take(len)?;
        Ok(b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Network {
        let a = Layer::new(
            Matrix::from_vec(3, 2, vec![0.1, -0.2, 0.3, 0.4, -0.5, 0.6]),
            vec![0.01, 0.02, -0.03],
            Activation::Relu,
        );
        let b = Layer::new(
            Matrix::from_vec(2, 3, vec![1.0, 2.0, 3.0, -1.0, -2.0, -3.0]),
            vec![0.5, -0.5],
            Activation::Softmax,
        );
        Network::new(vec![a, b])
    }

    #[test]
    fn round_trip_is_identical() {
        let net = sample();
        let bytes = net.to_bytes();
        assert_eq!(Network::from_bytes(&bytes).unwrap(), net);
    }

    #[test]
    fn header_layout_matches_the_spec() {
        let bytes = sample().to_bytes();
        assert_eq!(&bytes[0..4], b"DNN1");
        assert_eq!(&bytes[4..8], &3u32.to_le_bytes());
        assert_eq!(&bytes[8..12], &2u32.to_le_bytes());
        assert_eq!(&bytes[12..16], &3u32.to_le_bytes());
        assert_eq!(&bytes[16..20], &2u32.to_le_bytes());
        assert_eq!(&bytes[20..22], &[0, 2]);
        // 22 header bytes + (6 + 3 + 6 + 2) floats
        assert_eq!(bytes.len(), 22 + 17 * 4);
    }

    #[test]
    fn bad_input_is_rejected_not_panicked_on() {
        let good = sample().to_bytes();
        assert_eq!(Network::from_bytes(b"NOPE"), Err(FormatError::BadMagic));
        assert_eq!(Network::from_bytes(&good[..good.len() - 1]), Err(FormatError::UnexpectedEof));
        assert_eq!(Network::from_bytes(&good[..6]), Err(FormatError::UnexpectedEof));

        let mut extra = good.clone();
        extra.push(0);
        assert_eq!(Network::from_bytes(&extra), Err(FormatError::TrailingBytes));

        let mut bad_activation = good.clone();
        bad_activation[20] = 7;
        assert_eq!(Network::from_bytes(&bad_activation), Err(FormatError::UnknownActivation(7)));

        let mut huge = good;
        huge[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(Network::from_bytes(&huge), Err(FormatError::BadLayerSizes));
    }
}
