//! The guessed network must be a valid, reproducible network that does
//! clearly better than chance on the imagined digits it was built from.

use guess::guess::{build, build_with_seed, HIDDEN_1, HIDDEN_2};
use guess::raster::Canvas;
use guess::synth::varied_set;
use nn_core::preprocess::preprocess;
use nn_core::Network;

#[test]
fn it_is_an_ordinary_two_hidden_layer_network() {
    assert_eq!(build().layer_sizes(), vec![784, HIDDEN_1, HIDDEN_2, 10]);
}

#[test]
fn the_same_seed_gives_the_same_weights() {
    assert_eq!(build_with_seed(7), build_with_seed(7));
    assert_ne!(build_with_seed(7), build_with_seed(8));
}

#[test]
fn the_weights_file_round_trips() {
    let network = build();
    assert_eq!(Network::from_bytes(&network.to_bytes()).unwrap(), network);
}

#[test]
fn outputs_are_probabilities_even_for_an_empty_image() {
    let network = build();
    for input in [vec![0.0; 784], vec![1.0; 784]] {
        let output = network.output(&input);
        assert!(output.iter().all(|p| p.is_finite() && *p >= 0.0));
        assert!((output.iter().sum::<f32>() - 1.0).abs() < 1e-4);
    }
}

#[test]
fn it_beats_chance_by_a_wide_margin_on_imagined_digits() {
    // Chance is 10%. The guess reaches about 76%; this only guards against
    // breaking it, it is not a claim about real handwriting.
    let network = build();
    let samples = varied_set();
    let correct = samples.iter().filter(|s| network.classify(&s.pixels) == s.digit).count();
    let accuracy = correct as f32 / samples.len() as f32;
    assert!(accuracy > 0.6, "only {:.1}% of the imagined digits were recognised", accuracy * 100.0);
}

#[test]
fn a_vertical_line_is_a_one_and_a_ring_is_a_zero() {
    let network = build();
    let classify = |stroke: &[(f32, f32)]| {
        let mut canvas = Canvas::new(280, 280);
        canvas.stroke(stroke, 12.0);
        network.classify(&preprocess(&canvas.px, 280, 280))
    };
    assert_eq!(classify(&[(140.0, 40.0), (140.0, 240.0)]), 1);
    let ring: Vec<(f32, f32)> = (0..=36)
        .map(|i| {
            let a = (i as f32 * 10.0).to_radians();
            (140.0 + 65.0 * a.cos(), 140.0 + 95.0 * a.sin())
        })
        .collect();
    assert_eq!(classify(&ring), 0);
}
