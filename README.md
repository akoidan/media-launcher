# media-launcher

`media-launcher` scans a TV season folder and generates `NN.cmd`/`NN.bash` launch scripts to play each episode (including matching audio/subtitle tracks).

For each episode it finds, it creates:

- On Linux/macOS: `01.bash`, `02.bash`, ...
- On Windows: `01.cmd`, `02.cmd`, ...

Each script launches the episode in **mpv** or **vlc** players (and adds extra audio/subtitle tracks when present). So you don't longer need to manually add tracks for each episode.


## Tests

```sh
cargo test --test dump_fixture -- "D:\movies\[BD-Remux] Ore dake Level Up na Ken"
```


Will create `tests\fixtures\[BD-Remux] Ore dake Level Up na Ken.json`