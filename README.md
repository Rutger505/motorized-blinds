# Motorized Blinds

Project with 2 esp modules with motor attached to blinds, and a nRF52840 as remote

## Structure

This is a Cargo workspace. All crates share one `target/` directory and one `Cargo.lock`.

`driver` - Contains the code for the ESP modules that control the motors attached to the blinds.
`remote` - Contains the code for the nRF52840 remote that communicates with the ESP modules to control the blinds.
`protocol` - The radio frame format and commands, shared by `driver` and `remote`.
`case` - Contains the 3D model files for the case that houses the nRF52840 remote and ESP modules.

## Building

Each firmware crate builds for its own target. From the repository root:

```sh
cargo build-driver --release   # ESP32-C6, riscv32imac-unknown-none-elf
cargo build-remote --release   # nRF52840, thumbv7em-none-eabihf
cargo test-host                # hardware independent unit tests, on the host
```

Inside `driver/` or `remote/` a plain `cargo build` or `cargo run` picks the right target.
`cargo run-driver` and `cargo run-remote` flash and run through `probe-rs`.
