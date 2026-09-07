# espeak-synth

[![CI](https://github.com/glima31/espeak-synth/actions/workflows/ci.yml/badge.svg)](https://github.com/glima31/espeak-synth/actions/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/glima31/espeak-synth/blob/main/LICENSE)

A Rust library for text-to-speech synthesis with [eSpeak NG](https://github.com/espeak-ng/espeak-ng).

eSpeak NG is compiled from source during the build process and linked statically, removing the need to install it separately and allowing for seamless cross-platform builds.

## Usage

```toml
[dependencies]
espeak-synth = { git = "https://github.com/glima31/espeak-synth" }
```

```rust
use espeak_synth::{EspeakSynth, EspeakParam};

let espeak = EspeakSynth::default();

espeak.set_voice("German").unwrap();
espeak.set_parameter(EspeakParam::Pitch, 40).unwrap();
espeak.set_parameter(EspeakParam::Speed, 120).unwrap();

let mut buffer: Vec<i16> = Vec::new();
espeak.synthesize("Wir sind die Roboter", &mut buffer).unwrap();
```

`buffer` holds mono PCM samples at `espeak.sample_rate()`.

## Voice data

eSpeak NG cannot synthesize anything without its data directory, which holds the phoneme tables, language dictionaries and intonation data. That directory is compiled from source alongside the library, so you never have to obtain it separately and it always matches the version you linked against.

`EspeakSynth::default()` points at the copy the build produced, which is convenient during development and testing.

For anything you distribute, the data directory needs to be available at runtime and its path passed explicitly:

```rust
let espeak = EspeakSynth::new(&data_dir);
```

How it gets there is up to your packaging, whether that's next to the binary, in an OS-conventional location, or embedded in the executable and extracted on first run.

The build's copy is at `target/<profile>/build/espeak-sys-*/out/share/espeak-ng-data`.

## Workspace

| crate | |
|---|---|
| **espeak-sys** | Raw FFI bindings to eSpeak NG |
| **espeak-synth** (root crate) | Safe Rust API for speech synthesis, exposing `EspeakSynth` |

## Building

Requirements:

- [CMake](https://cmake.org/download/)
- [eSpeak NG build requirements](https://github.com/espeak-ng/espeak-ng/blob/master/docs/building.md)
- [bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)

After installing the requirements listed above, clone the repo with:

```sh
git clone --recurse-submodules https://github.com/glima31/espeak-synth.git
```

Then `cargo build`.

## Testing

eSpeak NG keeps global state, so tests can't share a process. [cargo-nextest](https://nexte.st/) runs each test in its own:

```sh
cargo nextest run
```

With plain `cargo test`, use `-- --test-threads=1`.

## License

This project is licensed under the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE) or <https://www.gnu.org/licenses/gpl-3.0.html>

eSpeak NG is itself GPL-3.0-or-later and is statically linked into anything built with this crate, so projects depending on it inherit that license.
