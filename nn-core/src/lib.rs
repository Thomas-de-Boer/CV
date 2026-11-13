//! A tiny neural network written from scratch.
//!
//! Zero dependencies, and no knowledge of WebAssembly or the browser.

pub mod activation;
pub mod matrix;
pub mod network;
pub mod preprocess;
pub mod serialize;
pub mod train;

pub use activation::Activation;
pub use matrix::Matrix;
pub use network::{Layer, Network};
pub use serialize::FormatError;
