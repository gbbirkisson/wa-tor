<img align="right" width="250" src="https://upload.wikimedia.org/wikipedia/commons/0/00/WaTor_rules.png">
<h1>Wa-Tor</h1>

This project is just a simple implementation of the [Wa-Tor](https://en.wikipedia.org/wiki/Wa-Tor) simulation. The simulation plays out in the terminal, so the terminal has to support emojis. The simulation is configured with some sane defaults, but feel free to clone it and play around with the constants at the top:

```rust
const FISH_BREED_INTERVAL: u8 = 10;
const SHARK_BREED_INTERVAL: u8 = 14;
const SHARK_STARVE_INTERVAL: u8 = 8;
const WRAP_WORLD: bool = true;
const MS_BETWEEN_CHRONON: u64 = 5;
```

## Running

```console
$ cargo run --release
```

## Test + Linting

```console
$ make
```
