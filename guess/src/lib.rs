//! A guess at the weights of a digit recogniser, made before any training.
//!
//! * [`shapes`] – how I picture each digit being written;
//! * [`synth`]  – those shapes drawn in many styles ("imagined digits");
//! * [`guess`]  – turns the imagined digits into weights for an ordinary,
//!   anonymous network (784 → 32 → 16 → 10);
//! * [`raster`] – a small stroke rasteriser and blur.

pub mod guess;
pub mod raster;
pub mod shapes;
pub mod synth;
