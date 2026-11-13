# Thomas de Boer — CV website

A static CV site with a small, working neural-network demo on the page. No
framework and no backend.

| Path          | What                                                              |
| ------------- | ----------------------------------------------------------------- |
| `index.html`  | The CV page.                                                      |
| `assets/`     | Its styles, scripts and fonts. `demo.js` is the small digit demo. |
| `ai/`         | The neural network: Rust source, weights and the full demo page (`ai/web/`). See [`ai/README.md`](ai/README.md). |

## Run it locally

```bash
cd ai
wasm-pack build nn-wasm --target web --release --out-dir ../web/pkg
cd ..
py -m http.server 8000        # WASM won't load from file://
```

Then open <http://localhost:8000>. The full demo is at
<http://localhost:8000/ai/web/>.

## Editing the content

- Text in both languages (English and Dutch) lives in `STRINGS` at the top of
  `assets/site.js`. Anything in `[square brackets]` is still a placeholder.
- The project cards are the repositories pinned on the GitHub profile, stored
  in `assets/projects.json`. Every deploy refreshes that file; to refresh it
  locally run `bash tools/update-projects.sh` (needs the GitHub CLI). The last
  card always links to the GitHub profile itself.

## Deploying

`.github/workflows/deploy.yml` tests the Rust code, builds the WASM module and
publishes the site to GitHub Pages on every push to `main`. In the repository
settings, set **Pages → Source** to **GitHub Actions**. All paths are
relative, so it works under `/<repo-name>/`.
