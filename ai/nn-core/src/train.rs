//! Training: random starting weights, backpropagation, and mini-batch
//! gradient descent with momentum.
//!
//! Supports the shape this project uses: ReLU hidden layers and a softmax
//! output trained with cross-entropy loss.

use crate::activation::Activation;
use crate::matrix::Matrix;
use crate::network::{Layer, Network};

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

    /// Uniform in 0..1 (never exactly 0).
    pub fn unit(&mut self) -> f32 {
        ((self.next() >> 40) as f32 + 1.0) / ((1u64 << 24) as f32 + 1.0)
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// Normal distribution, mean 0, standard deviation 1 (Box–Muller).
    pub fn normal(&mut self) -> f32 {
        (-2.0 * self.unit().ln()).sqrt() * (std::f32::consts::TAU * self.unit()).cos()
    }

    /// Fisher–Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.below(i + 1));
        }
    }
}

impl Network {
    /// A network with random weights (He initialisation: right for ReLU),
    /// zero biases, ReLU hidden layers and a softmax output.
    pub fn random(sizes: &[usize], seed: u64) -> Network {
        assert!(sizes.len() >= 2, "need at least an input and an output size");
        let mut rng = Rng::new(seed);
        let last = sizes.len() - 2;
        let layers = sizes
            .windows(2)
            .enumerate()
            .map(|(t, pair)| {
                let (inputs, outputs) = (pair[0], pair[1]);
                let scale = (2.0 / inputs as f32).sqrt();
                let weights = (0..inputs * outputs).map(|_| rng.normal() * scale).collect();
                let activation = if t == last { Activation::Softmax } else { Activation::Relu };
                Layer::new(Matrix::from_vec(outputs, inputs, weights), vec![0.0; outputs], activation)
            })
            .collect();
        Network::new(layers)
    }
}

/// One value per weight and bias of a network (gradients, or momentum).
#[derive(Debug, Clone, PartialEq)]
pub struct Gradients {
    pub weights: Vec<Vec<f32>>,
    pub biases: Vec<Vec<f32>>,
}

impl Gradients {
    pub fn zeros(network: &Network) -> Self {
        Gradients {
            weights: network.layers().iter().map(|l| vec![0.0; l.inputs() * l.outputs()]).collect(),
            biases: network.layers().iter().map(|l| vec![0.0; l.outputs()]).collect(),
        }
    }

    pub fn clear(&mut self) {
        self.weights.iter_mut().chain(self.biases.iter_mut()).for_each(|v| v.fill(0.0));
    }
}

/// Runs one example through the network, **adds** the gradient of its
/// cross-entropy loss to `gradients`, and returns `(loss, was it right?)`.
pub fn backprop(network: &Network, input: &[f32], label: usize, gradients: &mut Gradients) -> (f32, bool) {
    let layers = network.layers();
    let last = layers.len() - 1;
    assert_eq!(layers[last].activation, Activation::Softmax, "the output layer must be softmax");
    assert!(layers[..last].iter().all(|l| l.activation == Activation::Relu), "hidden layers must be ReLU");

    let activations = network.forward(input);
    let output = &activations[last];
    let loss = -output[label].max(1e-12).ln();
    let correct = crate::network::argmax(output) == label;

    // For softmax + cross-entropy, the error at the output is simply
    // (probabilities − the right answer).
    let mut delta: Vec<f32> = output.clone();
    delta[label] -= 1.0;

    for t in (0..=last).rev() {
        let below: &[f32] = if t == 0 { input } else { &activations[t - 1] };
        let n = below.len();
        for (j, d) in delta.iter().enumerate() {
            gradients.biases[t][j] += d;
            if *d != 0.0 {
                let row = &mut gradients.weights[t][j * n..(j + 1) * n];
                row.iter_mut().zip(below).for_each(|(g, a)| *g += d * a);
            }
        }
        if t > 0 {
            // Pass the error back through the weights; a ReLU neuron that
            // was off lets nothing through.
            let weights = &layers[t].weights;
            let mut previous = vec![0.0f32; n];
            for (j, d) in delta.iter().enumerate() {
                if *d != 0.0 {
                    previous.iter_mut().zip(weights.row(j)).for_each(|(p, w)| *p += d * w);
                }
            }
            previous.iter_mut().zip(below).for_each(|(p, a)| {
                if *a <= 0.0 {
                    *p = 0.0;
                }
            });
            delta = previous;
        }
    }
    (loss, correct)
}

/// Mini-batch gradient descent with momentum.
pub struct Trainer {
    pub learning_rate: f32,
    pub momentum: f32,
    gradients: Gradients,
    velocity: Gradients,
}

impl Trainer {
    pub fn new(network: &Network, learning_rate: f32, momentum: f32) -> Self {
        Trainer { learning_rate, momentum, gradients: Gradients::zeros(network), velocity: Gradients::zeros(network) }
    }

    /// Learns from one batch: every weight is nudged a little in the direction
    /// that would have made the answers more right. Returns
    /// `(summed loss, number answered correctly)` measured before the nudge.
    pub fn train_batch(&mut self, network: &mut Network, inputs: &[&[f32]], labels: &[usize]) -> (f32, usize) {
        assert_eq!(inputs.len(), labels.len());
        self.gradients.clear();
        let (mut loss, mut correct) = (0.0, 0);
        for (input, label) in inputs.iter().zip(labels) {
            let (l, ok) = backprop(network, input, *label, &mut self.gradients);
            loss += l;
            correct += ok as usize;
        }
        let step = self.learning_rate / inputs.len().max(1) as f32;
        for (t, layer) in network.layers_mut().iter_mut().enumerate() {
            let pairs = [
                (layer.weights.as_mut_slice(), &self.gradients.weights[t], &mut self.velocity.weights[t]),
                (layer.biases.as_mut_slice(), &self.gradients.biases[t], &mut self.velocity.biases[t]),
            ];
            for (values, gradient, velocity) in pairs {
                for ((w, g), v) in values.iter_mut().zip(gradient.iter()).zip(velocity.iter_mut()) {
                    *v = self.momentum * *v - step * g;
                    *w += *v;
                }
            }
        }
        (loss, correct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loss(network: &Network, input: &[f32], label: usize) -> f32 {
        -network.output(input)[label].ln()
    }

    #[test]
    fn random_networks_are_reproducible_and_well_formed() {
        let a = Network::random(&[784, 32, 16, 10], 1);
        assert_eq!(a, Network::random(&[784, 32, 16, 10], 1));
        assert_ne!(a, Network::random(&[784, 32, 16, 10], 2));
        assert_eq!(a.layer_sizes(), vec![784, 32, 16, 10]);
        let w = a.layers()[0].weights.as_slice();
        let mean = w.iter().sum::<f32>() / w.len() as f32;
        let spread = (w.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / w.len() as f32).sqrt();
        assert!(mean.abs() < 0.005, "mean = {mean}");
        assert!((spread - (2.0f32 / 784.0).sqrt()).abs() < 0.005, "spread = {spread}");
    }

    /// The backprop gradient must match "wiggle one weight and see how the
    /// loss changes" for every single weight and bias.
    #[test]
    fn gradients_match_numerical_differences() {
        let mut network = Network::random(&[5, 4, 3, 3], 11);
        // Non-zero biases, so they are tested properly too.
        let mut rng = Rng::new(5);
        for layer in network.layers_mut() {
            layer.biases.iter_mut().for_each(|b| *b = 0.3 * rng.normal());
        }
        let input = [0.9, 0.1, 0.5, 0.0, 0.7];
        let label = 2;

        let mut analytic = Gradients::zeros(&network);
        backprop(&network, &input, label, &mut analytic);

        let eps = 2e-3;
        let mut checked = 0;
        for t in 0..network.layers().len() {
            let n_weights = network.layers()[t].weights.as_slice().len();
            let n_biases = network.layers()[t].biases.len();
            for i in 0..n_weights + n_biases {
                let nudge = |amount: f32, network: &mut Network| {
                    let layer = &mut network.layers_mut()[t];
                    if i < n_weights {
                        layer.weights.as_mut_slice()[i] += amount;
                    } else {
                        layer.biases[i - n_weights] += amount;
                    }
                };
                nudge(eps, &mut network);
                let up = loss(&network, &input, label);
                nudge(-2.0 * eps, &mut network);
                let down = loss(&network, &input, label);
                nudge(eps, &mut network);
                let numeric = (up - down) / (2.0 * eps);
                let exact = if i < n_weights { analytic.weights[t][i] } else { analytic.biases[t][i - n_weights] };
                assert!(
                    (numeric - exact).abs() < 2e-3 + 0.03 * exact.abs(),
                    "layer {t}, parameter {i}: backprop {exact}, numerical {numeric}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 5 * 4 + 4 + 4 * 3 + 3 + 3 * 3 + 3);
        assert!(analytic.weights.iter().flatten().any(|g| g.abs() > 0.01), "gradient is suspiciously empty");
    }

    #[test]
    fn a_small_network_can_memorise_twenty_examples() {
        let mut rng = Rng::new(3);
        let inputs: Vec<Vec<f32>> = (0..20).map(|_| (0..12).map(|_| rng.unit()).collect()).collect();
        let labels: Vec<usize> = (0..20).map(|i| i % 4).collect();
        let mut network = Network::random(&[12, 16, 8, 4], 9);
        let mut trainer = Trainer::new(&network, 0.1, 0.9);
        let batch: Vec<&[f32]> = inputs.iter().map(Vec::as_slice).collect();

        let (first_loss, _) = trainer.train_batch(&mut network, &batch, &labels);
        let mut last = (first_loss, 0);
        for _ in 0..600 {
            last = trainer.train_batch(&mut network, &batch, &labels);
        }
        assert_eq!(last.1, 20, "should get all 20 right, loss went {first_loss} -> {}", last.0);
        assert!(last.0 < first_loss * 0.05);
    }

    #[test]
    fn training_is_deterministic() {
        let run = || {
            let mut network = Network::random(&[4, 5, 3], 2);
            let mut trainer = Trainer::new(&network, 0.05, 0.9);
            let inputs: [&[f32]; 2] = [&[1.0, 0.0, 0.5, 0.2], &[0.0, 1.0, 0.1, 0.9]];
            for _ in 0..10 {
                trainer.train_batch(&mut network, &inputs, &[0, 2]);
            }
            network
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn shuffle_keeps_every_item() {
        let mut items: Vec<usize> = (0..100).collect();
        Rng::new(4).shuffle(&mut items);
        assert_ne!(items, (0..100).collect::<Vec<_>>());
        items.sort();
        assert_eq!(items, (0..100).collect::<Vec<_>>());
    }
}
