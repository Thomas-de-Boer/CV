# Handwritten digit neural network (Rust → WebAssembly)

Draw a digit, watch a neural network light up layer by layer, see its answer.
The network is written from scratch in Rust (no ML libraries) and runs in the
browser as WebAssembly. The site is static; there is no backend.

The network is an ordinary one, **784 → 32 → 16 → 10**. The site ships two sets
of weights for it, with a switch at the top of the page:

| Weights           | How they were made                                         | MNIST test images (10,000) |
| ----------------- | ---------------------------------------------------------- | -------------------------- |
| **Claude-picked** | An experiment to see how good AI is at making AI: Claude chose every weight by reasoning, with no data and no training. | 39.7% |
| **Trained**       | Random start, then backpropagation on MNIST.                | 96.8% (trained on the other 60,000) |

The trained weights that ship were then retrained the same way on all 70,000
images, so there are no unseen MNIST images left to test them on. Your own
drawings are the remaining fair test (see below).

## Run it locally

```bash
cargo run --release -p guess                # writes web/weights-guess.bin (Claude-picked)
wasm-pack build nn-wasm --target web --release --out-dir ../web/pkg
cd .. && py -m http.server 8000             # WASM won't load from file://
```

Run the first two from this `ai/` folder; the server runs from the repository
root because the demo page shares its styles and fonts with the CV page
(`../assets/`). Then open <http://localhost:8000/ai/web/>. One-time setup:
`rustup target add wasm32-unknown-unknown` and `cargo install wasm-pack`.

## How the weights are guessed

No real handwriting, no gradient descent, nothing corrected by looking at
mistakes. `guess/src/guess.rs`:

1. `shapes.rs` describes how each digit is usually written; `synth.rs` draws
   those in many styles ("imagined digits"), averaged into one soft picture per digit.
2. Hidden layer 1: each neuron is a random mix of those pictures (some digits
   count for it, some against), seen through a random soft window, plus noise.
3. Hidden layer 2: each neuron is a random mix of how layer 1 typically reacts
   to each digit.
4. Output: each digit listens to the layer-2 neurons that react to it more than
   to other digits.

The random seed is fixed, so the same weights come out every time.

```bash
cargo run --release -p guess -- --report    # accuracy on the imagined digits
cargo test --release                        # everything
```

The report is measured on the same imagined digits the guess was built from,
so it says little about real handwriting.

## Training

Download MNIST once into `data/` (not committed) and unpack it:

```bash
mkdir data && cd data
for f in train-images-idx3-ubyte train-labels-idx1-ubyte t10k-images-idx3-ubyte t10k-labels-idx1-ubyte; do
  curl -fLO https://ossci-datasets.s3.amazonaws.com/mnist/$f.gz
  py -c "import gzip,sys; open(sys.argv[1],'wb').write(gzip.open(sys.argv[1]+'.gz').read())" $f
done
```

```bash
cargo run --release -p trainer                                  # train on 60,000, test on the other 10,000
cargo run --release -p trainer -- --all                         # train on all 70,000 -> web/weights-trained.bin
cargo run --release -p trainer -- --test web/weights-guess.bin  # score any weights file on the 10,000
```

Training starts from random weights (He initialisation, fixed seed) and runs
20 passes of mini-batch gradient descent (batch 32, momentum 0.9, learning rate
0.05 shrinking 15% per pass). Every image goes through the same crop / scale /
centre step as a drawing on the site. It takes about 25 seconds. The
backpropagation code is in `nn-core/src/train.rs` and is checked against
numerical gradients in the tests.

## Test it on your own handwriting

1. Open <http://localhost:8000/ai/web/?dev=1>.
2. Draw a digit, then press the key of the digit it *should* be (or click its
   button). The 28×28 image is saved with that label and the canvas clears.
3. Click **Export samples.json**, move the file to the project root, and run:

```bash
cargo run --release -p evaluate -- web/weights-guess.bin samples.json
cargo run --release -p evaluate -- web/weights-trained.bin samples.json
```

This prints the accuracy and a confusion matrix (which digits get mixed up).

## Layout

| Folder      | What                                                             |
| ----------- | ---------------------------------------------------------------- |
| `nn-core/`  | The network: matrix, activations, forward pass, training, file format, preprocessing. Zero dependencies. |
| `guess/`    | Generates the Claude-picked weights.                              |
| `trainer/`  | Trains on MNIST.                                                  |
| `evaluate/` | Accuracy + confusion matrix on saved samples.                     |
| `nn-wasm/`  | Thin `wasm-bindgen` wrapper around `nn-core`.                     |
| `web/`      | The site: drawing, visualisation (canvas 2D, no framework).       |

## Deploying

`../.github/workflows/deploy.yml` tests, builds the WASM module and publishes
the CV page with this demo under `ai/web/` to GitHub Pages on every push to
`main` (no training there: both weight files are committed). See the README
in the repository root.
