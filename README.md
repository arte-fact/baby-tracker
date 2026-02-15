# baby-tracker

A PWA to track baby feeding activity. Rust core compiled to WebAssembly.

## Features

- Track feedings: breast (left/right), bottle, and solid food
- Record amount (ml), duration (minutes), and notes
- View feeding history and summary statistics
- Installable as a PWA (works offline)
- Data stored locally in the browser (localStorage)

## Architecture

```
src/
  lib.rs         # WASM bindings (thin wrapper)
  tracker.rs     # Core API (testable on native)
  models.rs      # Domain models (Feeding, FeedingType)
  store.rs       # In-memory store with JSON serialization
web/
  index.html     # PWA shell
  js/app.js      # Frontend calling into WASM
  css/style.css  # Mobile-first styles
  sw.js          # Service worker for offline support
  manifest.json  # PWA manifest
```

## Development

```sh
# Run tests
cargo test

# Build WASM (requires wasm-pack)
wasm-pack build --target web --out-dir web/pkg
```

## Deployment

Push to `develop` branch. GitHub Actions will:
1. Run `cargo test`
2. Build WASM with `wasm-pack`
3. Deploy `web/` to GitHub Pages
