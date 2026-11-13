//! Trains 784 → 32 → 16 → 10 on MNIST, starting from random weights.
//!
//! ```text
//! cargo run --release -p trainer                 # train on 60,000, test on the other 10,000
//! cargo run --release -p trainer -- --all        # train on all 70,000 (no test left over)
//! cargo run --release -p trainer -- --test web/weights-guess.bin   # no training: score a file on the 10,000
//! ```
//!
//! Options: `--data <folder>` (default `data`), `--out <file>` (default
//! `web/weights-trained.bin`), `--epochs <n>` (default 20), `--seed <n>`.

mod mnist;

use mnist::Dataset;
use nn_core::train::{Rng, Trainer};
use nn_core::Network;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

const SIZES: [usize; 4] = [784, 32, 16, 10];
const BATCH: usize = 32;
const LEARNING_RATE: f32 = 0.05;
const LEARNING_RATE_DECAY: f32 = 0.85; // per epoch
const MOMENTUM: f32 = 0.9;

struct Options {
    data: PathBuf,
    out: PathBuf,
    all: bool,
    epochs: usize,
    seed: u64,
    /// Don't train: score this weights file on the 10,000 test images.
    test_only: Option<PathBuf>,
}

fn options() -> Result<Options, String> {
    let mut o = Options { data: "data".into(), out: "web/weights-trained.bin".into(), all: false, epochs: 20, seed: 1, test_only: None };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "--all" => o.all = true,
            "--data" => o.data = value("--data")?.into(),
            "--test" => o.test_only = Some(value("--test")?.into()),
            "--out" => o.out = value("--out")?.into(),
            "--epochs" => o.epochs = value("--epochs")?.parse().map_err(|_| "--epochs needs a number")?,
            "--seed" => o.seed = value("--seed")?.parse().map_err(|_| "--seed needs a number")?,
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    Ok(o)
}

/// `(accuracy, confusion[drawn][answered])`
fn test(network: &Network, data: &Dataset) -> (f32, [[usize; 10]; 10]) {
    let mut confusion = [[0usize; 10]; 10];
    for i in 0..data.len() {
        confusion[data.labels[i]][network.classify(data.image(i))] += 1;
    }
    let correct: usize = (0..10).map(|d| confusion[d][d]).sum();
    (correct as f32 / data.len().max(1) as f32, confusion)
}

fn print_confusion(confusion: &[[usize; 10]; 10]) {
    println!("\nconfusion matrix (rows = written digit, columns = what the network said)");
    println!("      {}", (0..10).map(|d| format!("{d:>5}")).collect::<String>());
    for (d, row) in confusion.iter().enumerate() {
        let n: usize = row.iter().sum();
        let cells: String = row.iter().map(|c| if *c == 0 { "    .".to_string() } else { format!("{c:>5}") }).collect();
        println!("  {d} | {cells}   {:>5.1}%", 100.0 * row[d] as f32 / n.max(1) as f32);
    }
}

fn run(o: &Options) -> Result<(), String> {
    println!("loading MNIST from {} ...", o.data.display());
    if let Some(path) = &o.test_only {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let network = Network::from_bytes(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        let held_out = mnist::load(&o.data, "t10k")?;
        let (accuracy, confusion) = test(&network, &held_out);
        println!("{} on the {} MNIST test images: {:.2}%", path.display(), held_out.len(), 100.0 * accuracy);
        print_confusion(&confusion);
        return Ok(());
    }
    let mut train = mnist::load(&o.data, "train")?;
    let held_out = mnist::load(&o.data, "t10k")?;
    let held_out = if o.all {
        train.append(held_out);
        None
    } else {
        Some(held_out)
    };
    match &held_out {
        Some(t) => println!("training on {} images, testing on {} it never sees", train.len(), t.len()),
        None => println!("training on all {} images (nothing left to test on)", train.len()),
    }

    let mut network = Network::random(&SIZES, o.seed);
    let mut trainer = Trainer::new(&network, LEARNING_RATE, MOMENTUM);
    let mut rng = Rng::new(o.seed ^ 0xD161_7A11);
    let mut order: Vec<usize> = (0..train.len()).collect();
    let started = Instant::now();
    let mut updates = 0usize;

    if let Some(t) = &held_out {
        println!("before training (random weights): {:.1}% on the test images", 100.0 * test(&network, t).0);
    }
    for epoch in 1..=o.epochs {
        rng.shuffle(&mut order);
        let (mut loss, mut correct) = (0.0f64, 0usize);
        for batch in order.chunks(BATCH) {
            let inputs: Vec<&[f32]> = batch.iter().map(|&i| train.image(i)).collect();
            let labels: Vec<usize> = batch.iter().map(|&i| train.labels[i]).collect();
            let (l, c) = trainer.train_batch(&mut network, &inputs, &labels);
            loss += l as f64;
            correct += c;
            updates += 1;
        }
        let mut line = format!(
            "epoch {epoch:>2}/{}  loss {:.4}  right while training {:.2}%",
            o.epochs,
            loss / train.len() as f64,
            100.0 * correct as f32 / train.len() as f32
        );
        if let Some(t) = &held_out {
            line += &format!("  test {:.2}%", 100.0 * test(&network, t).0);
        }
        println!("{line}  ({:.0}s)", started.elapsed().as_secs_f32());
        trainer.learning_rate *= LEARNING_RATE_DECAY;
    }

    let (train_accuracy, _) = test(&network, &train);
    println!("\naccuracy on the {} training images: {:.2}%", train.len(), 100.0 * train_accuracy);
    if let Some(t) = &held_out {
        let (accuracy, confusion) = test(&network, t);
        println!("accuracy on the {} test images:     {:.2}%", t.len(), 100.0 * accuracy);
        print_confusion(&confusion);
    }

    let parameters: usize = network.layers().iter().map(|l| l.inputs() * l.outputs() + l.outputs()).sum();
    println!(
        "\nsummary: {} images × {} passes = {} examples shown, {} weight updates of {} weights, batch {BATCH}, seed {}",
        train.len(),
        o.epochs,
        train.len() * o.epochs,
        updates,
        parameters,
        o.seed
    );

    if let Some(parent) = o.out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let bytes = network.to_bytes();
    std::fs::write(&o.out, &bytes).map_err(|e| format!("{}: {e}", o.out.display()))?;
    println!("wrote {} ({} bytes)", o.out.display(), bytes.len());
    Ok(())
}

fn main() -> ExitCode {
    match options().and_then(|o| run(&o)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
