# EdgeRunner – Kelly Criterion Calculator (Yew + WASM)

Fast, clean, and accurate Kelly calculator for real betting situations.

## Tech
- Rust + Yew (WASM)
- Trunk for build/bundle
- GitHub Pages for hosting (auto-deploy via Actions)

## Quick Start (local)

1. Install Rust and Trunk:
   - `rustup target add wasm32-unknown-unknown`
   - `cargo install trunk --locked`
2. Run dev server: `trunk serve`
3. Open: http://localhost:8080

## Build

- `trunk build --release`
- Output in `./dist`

## Deploy (GitHub Pages)

- Push to `main` (or `master`). The workflow builds and publishes to Pages.
- Repo path is assumed to be `/edgerunner/`. If deploying to a user/org root site (e.g. `<user>.github.io`), set `public_url = "/"` in `Trunk.toml` and change `<base href>` in `index.html` accordingly.

## MVP Scope

- Single-bet Kelly with fractional options (full/half/quarter)
- Edge metrics: EV per $1, implied probability, edge vs market
- Odds converter: Decimal, American, Fractional

## Next Up

- Multiple outcomes (mutually exclusive)
- Portfolio allocation across independent bets
- Visualizations: growth curves, risk profiles
- Presets and currency formatting

---

MIT-licensed. Built for speed and clarity.

