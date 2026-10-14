# PingDVD — agent instructions

## Stack
- Rust stable, Cargo, eframe/egui
- Entry point: `src/main.rs`
- Assets: `assets/`
- Settings: `pingdvd.settings.json` beside the executable, with a per-user fallback

## Commands
```bash
cargo run
cargo test
cargo build --release
cargo fmt --all -- --check
```

## CI
- GitHub Actions builds and tests Windows and macOS.
- Pushes to `Avalonia_dotnet_8` create a release; `v*` tags trigger CI.
