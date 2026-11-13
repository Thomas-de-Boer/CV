//! Guessing weights for 784 → 32 → 16 → 10 (ReLU, ReLU, softmax).
//!
//! Nothing here is training: no real handwriting, no gradient descent, and no
//! weight is ever adjusted by looking at a mistake. The guess is built from
//! "imagined digits" (the shapes in [`crate::shapes`], drawn in many styles):
//!
//! 1. average the imagined digits into one soft picture per digit;
//! 2. hidden layer 1: every neuron is a *random* mix of those pictures
//!    (some digits count for, some against), seen through a random soft
//!    window, plus a little noise;
//! 3. hidden layer 2: every neuron is a random mix of how layer 1 typically
//!    reacts to each digit;
//! 4. output: each digit listens to the layer-2 neurons that typically react
//!    to it more than to the other digits.
//!
//! The neurons have no names and no assigned jobs.

use crate::raster::{blur, dot};
use crate::synth::{varied_set, Sample};
use nn_core::{Activation, Layer, Matrix, Network};

pub const HIDDEN_1: usize = 32;
pub const HIDDEN_2: usize = 16;
pub const SEED: u64 = 2026;

/// Fraction of imagined digits a hidden neuron should fire for.
const FIRING_RATE: f32 = 0.35;
/// Random noise added to every weight, relative to the largest weight.
const NOISE: f32 = 0.04;
/// How sharply the output turns evidence into probabilities.
const CONFIDENCE: f32 = 2.5;

/// Small deterministic random number generator (xorshift64*).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in 0..1.
    pub fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.unit()
    }

    pub fn below(&mut self, n: usize) -> usize {
        ((self.unit() * n as f32) as usize).min(n - 1)
    }

    /// Roughly normal, mean 0, spread 1.
    pub fn noise(&mut self) -> f32 {
        (0..4).map(|_| self.unit()).sum::<f32>() * 1.73 - 3.46
    }
}

/// Average of `vectors[i]` per digit, with the overall average subtracted:
/// "what is special about each digit".
fn digit_signatures(vectors: &[Vec<f32>], samples: &[Sample]) -> Vec<Vec<f32>> {
    let len = vectors[0].len();
    let mut sums = vec![vec![0.0f32; len]; 10];
    let mut counts = [0.0f32; 10];
    for (v, s) in vectors.iter().zip(samples) {
        counts[s.digit] += 1.0;
        sums[s.digit].iter_mut().zip(v).for_each(|(a, b)| *a += b);
    }
    for (sum, count) in sums.iter_mut().zip(counts) {
        sum.iter_mut().for_each(|v| *v /= count.max(1.0));
    }
    let overall: Vec<f32> = (0..len).map(|i| sums.iter().map(|s| s[i]).sum::<f32>() / 10.0).collect();
    for sum in sums.iter_mut() {
        sum.iter_mut().zip(&overall).for_each(|(v, o)| *v -= o);
    }
    sums
}

/// A random mix of the digit signatures: a few digits count for the neuron,
/// a few against. `must_like` guarantees every digit has neurons on its side.
fn random_mix(signatures: &[Vec<f32>], must_like: usize, rng: &mut Rng) -> Vec<f32> {
    let mut coefficients = [0.0f32; 10];
    coefficients[must_like] = rng.range(0.6, 1.0);
    for _ in 0..rng.below(3) {
        coefficients[rng.below(10)] = rng.range(0.4, 1.0);
    }
    for _ in 0..1 + rng.below(3) {
        let d = rng.below(10);
        if d != must_like {
            coefficients[d] = -rng.range(0.4, 1.0);
        }
    }
    let mut mix = vec![0.0; signatures[0].len()];
    for (signature, c) in signatures.iter().zip(coefficients) {
        mix.iter_mut().zip(signature).for_each(|(m, s)| *m += c * s);
    }
    mix
}

fn add_noise(weights: &mut [f32], rng: &mut Rng) {
    let largest = weights.iter().fold(0.0f32, |m, w| m.max(w.abs()));
    weights.iter_mut().for_each(|w| *w += NOISE * largest * rng.noise());
}

/// Rescales a neuron so its reactions to the imagined digits have spread 1,
/// and returns the bias that makes it fire for `FIRING_RATE` of them.
fn calibrate(weights: &mut [f32], inputs: &[Vec<f32>]) -> f32 {
    let mut reactions: Vec<f32> = inputs.iter().map(|x| dot(weights, x)).collect();
    let mean = reactions.iter().sum::<f32>() / reactions.len() as f32;
    let spread = (reactions.iter().map(|r| (r - mean).powi(2)).sum::<f32>() / reactions.len() as f32).sqrt().max(1e-6);
    weights.iter_mut().for_each(|w| *w /= spread);
    reactions.iter_mut().for_each(|r| *r /= spread);
    reactions.sort_by(|a, b| a.total_cmp(b));
    let cut = ((1.0 - FIRING_RATE) * reactions.len() as f32) as usize;
    -reactions[cut.min(reactions.len() - 1)]
}

fn to_layer(rows: Vec<Vec<f32>>, biases: Vec<f32>, activation: Activation) -> Layer {
    let mut weights = Matrix::zeros(rows.len(), rows[0].len());
    for (i, row) in rows.iter().enumerate() {
        weights.row_mut(i).copy_from_slice(row);
    }
    Layer::new(weights, biases, activation)
}

fn run(layer: &Layer, inputs: &[Vec<f32>]) -> Vec<Vec<f32>> {
    inputs.iter().map(|x| layer.forward(x)).collect()
}

pub fn build() -> Network {
    build_with_seed(SEED)
}

pub fn build_with_seed(seed: u64) -> Network {
    let mut rng = Rng::new(seed);
    let samples = varied_set();
    let images: Vec<Vec<f32>> = samples.iter().map(|s| s.pixels.clone()).collect();

    // 1. One soft picture per digit.
    let pictures: Vec<Vec<f32>> = digit_signatures(&images, &samples).iter().map(|p| blur(p, 28, 28, 1.2)).collect();

    // 2. Hidden layer 1: random mixes of the pictures, each seen through a
    //    random soft window so a neuron cares about one region of the image.
    let (mut rows, mut biases) = (Vec::new(), Vec::new());
    for j in 0..HIDDEN_1 {
        let mut weights = random_mix(&pictures, j % 10, &mut rng);
        let (cx, cy, size) = (rng.range(7.0, 21.0), rng.range(6.0, 22.0), rng.range(4.5, 9.0));
        for (i, w) in weights.iter_mut().enumerate() {
            let d2 = ((i % 28) as f32 + 0.5 - cx).powi(2) + ((i / 28) as f32 + 0.5 - cy).powi(2);
            *w *= (-d2 / (2.0 * size * size)).exp();
        }
        add_noise(&mut weights, &mut rng);
        biases.push(calibrate(&mut weights, &images));
        rows.push(weights);
    }
    let layer1 = to_layer(rows, biases, Activation::Relu);
    let hidden1 = run(&layer1, &images);

    // 3. Hidden layer 2: random mixes of how layer 1 reacts to each digit.
    let reactions1 = digit_signatures(&hidden1, &samples);
    let (mut rows, mut biases) = (Vec::new(), Vec::new());
    for k in 0..HIDDEN_2 {
        let mut weights = random_mix(&reactions1, k % 10, &mut rng);
        add_noise(&mut weights, &mut rng);
        biases.push(calibrate(&mut weights, &hidden1));
        rows.push(weights);
    }
    let layer2 = to_layer(rows, biases, Activation::Relu);
    let hidden2 = run(&layer2, &hidden1);

    // 4. Output: each digit listens to the neurons that react to it more
    //    than to the other digits.
    let reactions2 = digit_signatures(&hidden2, &samples);
    let (mut rows, mut biases) = (Vec::new(), Vec::new());
    for signature in &reactions2 {
        let mut weights = signature.clone();
        add_noise(&mut weights, &mut rng);
        let reactions: Vec<f32> = hidden2.iter().map(|h| dot(&weights, h)).collect();
        let mean = reactions.iter().sum::<f32>() / reactions.len() as f32;
        let spread = (reactions.iter().map(|r| (r - mean).powi(2)).sum::<f32>() / reactions.len() as f32).sqrt().max(1e-6);
        weights.iter_mut().for_each(|w| *w *= CONFIDENCE / spread);
        biases.push(-mean * CONFIDENCE / spread);
        rows.push(weights);
    }
    let layer3 = to_layer(rows, biases, Activation::Softmax);

    Network::new(vec![layer1, layer2, layer3])
}
