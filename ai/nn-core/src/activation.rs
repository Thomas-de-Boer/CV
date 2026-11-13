//! Activation functions, applied in place to a layer's values.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    Relu,
    Sigmoid,
    Softmax,
}

impl Activation {
    /// The code used in the weights file.
    pub fn code(self) -> u8 {
        match self {
            Activation::Relu => 0,
            Activation::Sigmoid => 1,
            Activation::Softmax => 2,
        }
    }

    pub fn from_code(code: u8) -> Option<Activation> {
        match code {
            0 => Some(Activation::Relu),
            1 => Some(Activation::Sigmoid),
            2 => Some(Activation::Softmax),
            _ => None,
        }
    }

    pub fn apply(self, values: &mut [f32]) {
        match self {
            Activation::Relu => values.iter_mut().for_each(|v| *v = v.max(0.0)),
            Activation::Sigmoid => values.iter_mut().for_each(|v| *v = 1.0 / (1.0 + (-*v).exp())),
            Activation::Softmax => softmax(values),
        }
    }
}

/// Softmax, stable for large inputs: the maximum is subtracted before `exp`.
fn softmax(values: &mut [f32]) {
    if values.is_empty() {
        return;
    }
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0;
    for v in values.iter_mut() {
        *v = (*v - max).exp();
        sum += *v;
    }
    for v in values.iter_mut() {
        *v /= sum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relu_clamps_negatives() {
        let mut v = [-2.0, 0.0, 3.5];
        Activation::Relu.apply(&mut v);
        assert_eq!(v, [0.0, 0.0, 3.5]);
    }

    #[test]
    fn sigmoid_is_half_at_zero_and_saturates() {
        let mut v = [0.0, 20.0, -20.0];
        Activation::Sigmoid.apply(&mut v);
        assert!((v[0] - 0.5).abs() < 1e-6);
        assert!(v[1] > 0.999 && v[2] < 0.001);
    }

    #[test]
    fn softmax_sums_to_one_and_keeps_order() {
        let mut v = [1.0, 2.0, 3.0];
        Activation::Softmax.apply(&mut v);
        assert!((v.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(v[0] < v[1] && v[1] < v[2]);
    }

    #[test]
    fn softmax_is_stable_for_large_inputs() {
        let mut v = [1000.0, 1001.0, 999.0];
        Activation::Softmax.apply(&mut v);
        assert!(v.iter().all(|x| x.is_finite()));
        assert!((v.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(v[1] > v[0] && v[0] > v[2]);
    }

    #[test]
    fn codes_round_trip() {
        for a in [Activation::Relu, Activation::Sigmoid, Activation::Softmax] {
            assert_eq!(Activation::from_code(a.code()), Some(a));
        }
        assert_eq!(Activation::from_code(9), None);
    }
}
