//! Runs a weights file against saved samples and prints accuracy and a
//! confusion matrix.
//!
//! ```text
//! cargo run --release -p evaluate -- web/weights-handcrafted.bin samples.json
//! ```
//!
//! `samples.json` is what the site exports in dev mode (`?dev=1`):
//! `{ "samples": [ { "label": 3, "pixels": [784 numbers in 0..1] }, ... ] }`

use nn_core::network::argmax;
use nn_core::Network;
use serde_json::Value;
use std::process::ExitCode;

struct Sample {
    label: usize,
    pixels: Vec<f32>,
}

fn load_samples(text: &str, input_size: usize) -> Result<Vec<Sample>, String> {
    let json: Value = serde_json::from_str(text).map_err(|e| format!("not valid JSON: {e}"))?;
    let list = json
        .get("samples")
        .unwrap_or(&json)
        .as_array()
        .ok_or("expected { \"samples\": [...] } or a plain array")?;
    let mut samples = Vec::with_capacity(list.len());
    for (i, item) in list.iter().enumerate() {
        let label = item.get("label").and_then(Value::as_u64).ok_or(format!("sample {i}: missing \"label\""))? as usize;
        if label > 9 {
            return Err(format!("sample {i}: label {label} is not a digit"));
        }
        let pixels: Vec<f32> = item
            .get("pixels")
            .and_then(Value::as_array)
            .ok_or(format!("sample {i}: missing \"pixels\""))?
            .iter()
            .map(|v| v.as_f64().map(|f| f as f32).ok_or(format!("sample {i}: pixels must be numbers")))
            .collect::<Result<_, _>>()?;
        if pixels.len() != input_size {
            return Err(format!("sample {i}: {} pixels, the network expects {input_size}", pixels.len()));
        }
        samples.push(Sample { label, pixels });
    }
    Ok(samples)
}

/// `confusion[drawn][answered]`
fn confusion_matrix(network: &Network, samples: &[Sample]) -> [[usize; 10]; 10] {
    let mut confusion = [[0usize; 10]; 10];
    for s in samples {
        confusion[s.label][argmax(&network.output(&s.pixels)).min(9)] += 1;
    }
    confusion
}

fn print_report(confusion: &[[usize; 10]; 10]) {
    let total: usize = confusion.iter().flatten().sum();
    let correct: usize = (0..10).map(|d| confusion[d][d]).sum();
    println!("accuracy: {correct}/{total} ({:.1}%)", 100.0 * correct as f32 / total.max(1) as f32);
    println!();
    println!("confusion matrix (rows = drawn digit, columns = what the network said)");
    println!("      {}", (0..10).map(|d| format!("{d:>4}")).collect::<String>());
    for (d, row) in confusion.iter().enumerate() {
        let n: usize = row.iter().sum();
        let cells: String = row.iter().map(|c| if *c == 0 { "   .".to_string() } else { format!("{c:>4}") }).collect();
        let pct = if n == 0 { "    -".to_string() } else { format!("{:>4.0}%", 100.0 * row[d] as f32 / n as f32) };
        println!("  {d} | {cells}   {pct}  ({n} samples)");
    }

    let mut mixups: Vec<(usize, usize, usize)> = Vec::new();
    for drawn in 0..10 {
        for said in 0..10 {
            if drawn != said && confusion[drawn][said] > 0 {
                mixups.push((confusion[drawn][said], drawn, said));
            }
        }
    }
    mixups.sort_by(|a, b| b.cmp(a));
    if !mixups.is_empty() {
        println!();
        println!("most common mix-ups:");
        for (count, drawn, said) in mixups.iter().take(8) {
            println!("  a drawn {drawn} read as {said}: {count}×");
        }
    }
}

fn run(weights_path: &str, samples_path: &str) -> Result<(), String> {
    let bytes = std::fs::read(weights_path).map_err(|e| format!("{weights_path}: {e}"))?;
    let network = Network::from_bytes(&bytes).map_err(|e| format!("{weights_path}: {e}"))?;
    let text = std::fs::read_to_string(samples_path).map_err(|e| format!("{samples_path}: {e}"))?;
    let samples = load_samples(&text, network.input_size()).map_err(|e| format!("{samples_path}: {e}"))?;
    if samples.is_empty() {
        return Err(format!("{samples_path}: no samples"));
    }
    println!("weights: {weights_path}  layers: {:?}", network.layer_sizes());
    println!("samples: {samples_path}  ({})", samples.len());
    print_report(&confusion_matrix(&network, &samples));
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: evaluate <weights.bin> <samples.json>");
        return ExitCode::from(2);
    }
    match run(&args[0], &args[1]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nn_core::{Activation, Layer, Matrix};

    /// 2 inputs: answers 0 if the first is larger, else 1.
    fn tiny() -> Network {
        let mut w = Matrix::zeros(10, 2);
        w.set(0, 0, 1.0);
        w.set(1, 1, 1.0);
        Network::new(vec![Layer::new(w, vec![0.0; 10], Activation::Softmax)])
    }

    #[test]
    fn loads_both_json_layouts() {
        let wrapped = r#"{ "samples": [ { "label": 1, "pixels": [0, 1] } ] }"#;
        let bare = r#"[ { "label": 0, "pixels": [1.0, 0.25] } ]"#;
        assert_eq!(load_samples(wrapped, 2).unwrap()[0].label, 1);
        assert_eq!(load_samples(bare, 2).unwrap()[0].pixels, vec![1.0, 0.25]);
    }

    #[test]
    fn rejects_bad_samples() {
        assert!(load_samples("nope", 2).is_err());
        assert!(load_samples(r#"[{ "label": 12, "pixels": [0, 1] }]"#, 2).is_err());
        assert!(load_samples(r#"[{ "label": 1, "pixels": [0] }]"#, 2).is_err());
        assert!(load_samples(r#"[{ "pixels": [0, 1] }]"#, 2).is_err());
    }

    #[test]
    fn confusion_counts_drawn_versus_said() {
        let samples = vec![
            Sample { label: 0, pixels: vec![1.0, 0.0] }, // said 0, right
            Sample { label: 1, pixels: vec![0.0, 1.0] }, // said 1, right
            Sample { label: 1, pixels: vec![1.0, 0.0] }, // said 0, wrong
        ];
        let c = confusion_matrix(&tiny(), &samples);
        assert_eq!(c[0][0], 1);
        assert_eq!(c[1][1], 1);
        assert_eq!(c[1][0], 1);
    }
}
