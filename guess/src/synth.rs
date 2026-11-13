//! Imagined digits, drawn in code: the only "examples" the guess is based on.
//!
//! These are the shapes from [`crate::shapes`] drawn with different widths,
//! slants, brush sizes, sizes and a little wobble, then pushed through the
//! same preprocessing as a real drawing. They are *not* real handwriting.

use crate::shapes::{render, shape, variant_count, Style, CANVAS};
use nn_core::preprocess::preprocess;

pub struct Sample {
    pub digit: usize,
    pub variant: usize,
    pub style: Style,
    /// 28×28, as the network sees it.
    pub pixels: Vec<f32>,
}

pub fn draw(digit: usize, variant: usize, style: &Style) -> Vec<f32> {
    preprocess(&render(&shape(digit, variant), style), CANVAS, CANVAS)
}

/// The textbook version of each way of writing a digit.
pub fn clean_set() -> Vec<Sample> {
    let style = Style::default();
    let mut out = Vec::new();
    for digit in 0..10 {
        for variant in 0..variant_count(digit) {
            out.push(Sample { digit, variant, style, pixels: draw(digit, variant, &style) });
        }
    }
    out
}

/// Every digit in many styles (width × slant × size × brush × wobble).
pub fn varied_set() -> Vec<Sample> {
    let mut out = Vec::new();
    let mut seed = 0u32;
    for digit in 0..10 {
        for variant in 0..variant_count(digit) {
            for widen in [0.8, 1.0, 1.25] {
                for slant in [-0.08, 0.08, 0.22] {
                    // The site's brush has a fixed size, so small drawings
                    // come out thick and large ones thin.
                    for height in [110.0, 160.0, 220.0] {
                        for radius in [11.0, 13.0] {
                            for wobble in [0.0, 0.025] {
                                seed += 1;
                                let style = Style {
                                    height,
                                    widen,
                                    slant,
                                    radius,
                                    rotate: ((seed % 5) as f32 - 2.0) * 0.03,
                                    wobble,
                                    seed,
                                    centre: (110.0 + (seed % 7) as f32 * 10.0, 120.0 + (seed % 4) as f32 * 12.0),
                                };
                                out.push(Sample { digit, variant, style, pixels: draw(digit, variant, &style) });
                            }
                        }
                    }
                }
            }
        }
    }
    out
}
