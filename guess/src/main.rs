//! Writes the guessed weights.
//!
//! ```text
//! cargo run --release -p guess               # write web/weights-guess.bin
//! cargo run --release -p guess -- --report   # how it does on imagined digits
//! ```

use guess::guess::build;
use guess::synth::{clean_set, varied_set, Sample};
use nn_core::Network;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let network = build();
    match args.first().map(String::as_str) {
        None => write_file(&network, Path::new("web")),
        Some("--out") => write_file(&network, Path::new(args.get(1).expect("--out needs a folder"))),
        Some("--report") => {
            println!("NOTE: these are the same imagined digits the guess was built from,");
            println!("so this says little about real handwriting.");
            accuracy(&network, &clean_set(), "textbook digits");
            let varied = varied_set();
            accuracy(&network, &varied, "varied imagined digits");
            by_style(&network, &varied);
        }
        Some(other) => {
            eprintln!("unknown argument {other:?} (try --report or --out <folder>)");
            std::process::exit(2);
        }
    }
}

fn write_file(network: &Network, folder: &Path) {
    std::fs::create_dir_all(folder).expect("create output folder");
    let path = folder.join("weights-guess.bin");
    let bytes = network.to_bytes();
    std::fs::write(&path, &bytes).expect("write weights");
    println!("layers: {:?}", network.layer_sizes());
    println!("wrote {} ({} bytes)", path.display(), bytes.len());
}

fn accuracy(network: &Network, samples: &[Sample], title: &str) {
    let mut confusion = [[0usize; 10]; 10];
    for s in samples {
        confusion[s.digit][network.classify(&s.pixels)] += 1;
    }
    let correct: usize = (0..10).map(|d| confusion[d][d]).sum();
    println!("\n{title}: {correct}/{} correct ({:.1}%)", samples.len(), 100.0 * correct as f32 / samples.len() as f32);
    println!("rows = drawn digit, columns = what the network said");
    println!("      {}", (0..10).map(|d| format!("{d:>5}")).collect::<String>());
    for (d, row) in confusion.iter().enumerate() {
        let total: usize = row.iter().sum();
        let cells: String = row.iter().map(|c| if *c == 0 { "    .".to_string() } else { format!("{c:>5}") }).collect();
        println!("  {d} | {cells}   {:>5.1}%", 100.0 * row[d] as f32 / total.max(1) as f32);
    }
}

/// Which kinds of handwriting the network struggles with.
fn by_style(network: &Network, samples: &[Sample]) {
    type Key = fn(&Sample) -> String;
    let factors: [(&str, Key); 4] = [
        ("width", |s| format!("×{}", s.style.widen)),
        ("slant", |s| format!("{}", s.style.slant)),
        ("stroke thickness (28×28 px)", |s| format!("{:.1}", 2.0 * s.style.radius * 20.0 / s.style.height)),
        ("way of writing", |s| format!("{} v{}", s.digit, s.variant)),
    ];
    println!("\naccuracy by style:");
    for (title, key) in factors {
        let mut groups: Vec<(String, usize, usize)> = Vec::new();
        for s in samples {
            let k = key(s);
            let ok = (network.classify(&s.pixels) == s.digit) as usize;
            match groups.iter_mut().find(|g| g.0 == k) {
                Some(g) => {
                    g.1 += ok;
                    g.2 += 1;
                }
                None => groups.push((k, ok, 1)),
            }
        }
        let cells: Vec<String> =
            groups.iter().map(|(k, ok, n)| format!("{k}: {:.0}%", 100.0 * *ok as f32 / *n as f32)).collect();
        println!("  {title:<28} {}", cells.join("   "));
    }
}
