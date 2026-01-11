## Tests

```sh
cargo test --test mock_fs_tests
```


### Generate test fixtures

```sh
cargo run -p xtask -- dump-fixture "D:\movies\Kaijuu 8 Gou TV-2 [WEB-DL 1080p]"
```

Will create `tests\fixtures\Kimetsu.no.Yaiba.Katanakaji.no.Sato.hen.WEB-DL.1080p`


## Lint


```sh
cargo fmt-check
cargo lint
```

## Autoformat

```sh
cargo fmt --all -- --check
```
