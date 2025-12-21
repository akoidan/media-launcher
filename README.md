# media-launcher

`media-launcher` scans a TV season folder and generates `NN.cmd`/`NN.bash` launch scripts to play each episode (including matching audio/subtitle tracks).

For each episode it finds, it creates:

- On Linux/macOS: `01.bash`, `02.bash`, ...
- On Windows: `01.cmd`, `02.cmd`, ...

Each script launches the episode in **mpv** or **vlc** players (and adds extra audio/subtitle tracks when present). So you don't longer need to manually add tracks for each episode.


## Usage

 - Download executable application file from [releases](https://github.com/akoidan/media-launcher/releases)
 - Run it either by opening it, either passing folder with TV series path as first argument
 - It will generate launch scripts in the video folder

For archlinux you can also install it via `yay -S media-launcher` or `paru -S media-launcher`

## Tests

```sh
cargo test --test mock_fs_tests
```


### Generate test fixtures

```sh
cargo run -p xtask -- dump-fixture "D:\movies\Kimetsu.no.Yaiba.Katanakaji.no.Sato.hen.WEB-DL.1080p"
```

Will create `tests\fixtures\Kimetsu.no.Yaiba.Katanakaji.no.Sato.hen.WEB-DL.1080p`


## Lint

If you want formatting/linting locally:

```sh
rustup component add rustfmt clippy
```