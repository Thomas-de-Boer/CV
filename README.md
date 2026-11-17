# Thomas de Boer: CV website

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
  `assets/site.js`. Change it there: the text in `index.html` is only what
  shows before the script runs, and is replaced by `STRINGS`.
- The project cards are the repositories pinned on the GitHub profile, stored
  in `assets/projects.json`. Every deploy refreshes that file; to refresh it
  locally run `bash tools/update-projects.sh` (needs the GitHub CLI). The last
  card always links to the GitHub profile itself. To give a project a picture,
  upload one on GitHub under the repository's **Settings → Social preview**
  (1280×640 works best); it appears on the card after the next deploy.
- `assets/og-image.png` is the picture shown when the site's link is shared;
  its source is `tools/og-image.html`. `assets/apple-touch-icon.png` comes from
  `tools/touch-icon.html` the same way.
- To add a photo: put a square photo (at least 600×600) at `assets/photo.jpg`
  and remove the comment markers around `hero-photo` in `index.html`.
- Headings use Archivo (`assets/fonts/archivo-latin.woff2`, SIL Open Font
  License in `assets/fonts/OFL.txt`); everything else uses the system fonts.

## Deploying

`.github/workflows/deploy.yml` tests the Rust code, builds the WASM module and
publishes the site to GitHub Pages on every push to `main`. In the repository
settings, set **Pages → Source** to **GitHub Actions**. All paths are
relative, so it works under `/<repo-name>/`.
