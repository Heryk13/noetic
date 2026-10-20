# Noetic

Open-source editor and runtime, written in Rust + Bevy, for game animals that feel alive: they have needs and personalities, perceive the world with limited senses, react to danger, live in groups, mark territory, fight and follow a day/night routine.

Behavior is built and tested in a 2D editor, exported to a `.noetic.ron` file and loaded by the game through the `noetic` crate.

> Status: milestone M1 (Core) is done. `noetic_core` ticks needs, picks an activity by Utility score
> and runs it step by step, deterministic from a seed and covered by unit tests and a golden
> determinism test. There is no Bevy plugin or editor yet.

## Development

- `just check`: formatting, clippy and every test
- `just fix`: apply clippy fixes and format the code
- `just test`: the tests only

## Documentation

Everything is in [docs/](docs/README.md): vision, requirements definition, basic design, external design, internal design, detailed design and the clickable prototype.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
