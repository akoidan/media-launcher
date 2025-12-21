# media-launcher

`media-launcher` scans a directory containing episode media files and generates one launch script per episode.

For each episode it finds, it creates:

- On Linux/macOS: `01.bash`, `02.bash`, ...
- On Windows: `01.cmd`, `02.cmd`, ...

Each script launches the episode in **mpv** (and adds extra audio/subtitle tracks when present).

## What it does

Given a root folder, it recursively scans files and groups them by episode number based on the filename:

- Matches `E01`, `E02`, ... (pattern `E(\d\d)`)
- Or any two-digit number `01`, `02`, ... (pattern `\d\d`)

Supported file types:

- `*.mkv` (video)
- `*.mka` (additional audio tracks)
- `*.ass` (subtitles)
- `*.ttf` (fonts are allowed but ignored when grouping)

It also detects a fonts directory (a folder whose name contains `font` or `шрифты`) and, if found, adds it to the command via `--sub-fonts-dir`.

## Requirements

- **Rust** (only needed if you build locally)
- **mpv** installed and available in `PATH`

Notes:

- On non-Windows platforms the generated scripts are made executable.
- The tool writes scripts into the provided `root_dir` and may overwrite existing `NN.bash` / `NN.cmd` files.

## Run locally

From the repository root:

```bash
cargo run -- <root_dir>
```

Select a player (defaults to `mpv`):

```bash
cargo run -- <root_dir> --player mpv
```

```bash
cargo run -- <root_dir> --player vlc
```

Example:

```bash
cargo run -- "/media/Shows/MyShow/Season 01"
```

## Development (formatting & linting)

This repo includes `rust-toolchain.toml` which pins the toolchain channel and requests the `rustfmt` and `clippy` components.

Local setup (requires `rustup`):

```bash
rustup toolchain install stable
```

Format check:

```bash
cargo fmt --all -- --check
```

Lint (Clippy):

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

If you have a rustup-managed toolchain, you can also use the provided cargo aliases:

```bash
cargo fmt-check
cargo lint
```

## Build a release binary:

```bash
cargo build --release
```

Binary location:

- Linux/macOS: `target/release/media-launcher`
- Windows: `target\release\media-launcher.exe`

## Use from GitHub Releases

1. Go to the repository **Releases** page.
2. Download the archive for your OS (Linux or Windows).
3. Extract it.
4. Run it, passing the folder you want to scan:

Linux/macOS:

```bash
./media-launcher /path/to/episode-folder
```

With VLC:

```bash
./media-launcher /path/to/episode-folder --player vlc
```

Windows (PowerShell):

```powershell
.\media-launcher.exe C:\Path\To\EpisodeFolder
```

With VLC:

```powershell
.\media-launcher.exe C:\Path\To\EpisodeFolder --player vlc
```

After running, you’ll find generated `NN.bash` / `NN.cmd` scripts in that folder; run the script for the episode you want to watch.

## Example folder layout

```
Season 01/
  E01.mkv
  E01.ass
  E01.mka
  E02.mkv
  fonts/
    SomeFont.ttf
```

Running `media-launcher` on `Season 01/` will generate `01.bash`/`01.cmd`, `02.bash`/`02.cmd`, etc.



## Tests

```sh
cargo run --bin dump_fixture -- "D:\movies\[BD-Remux] Ore dake Level Up na Ken"
```
Will create `tests\fixtures\[BD-Remux] Ore dake Level Up na Ken.json`