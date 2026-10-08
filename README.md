# Motorized Blinds

Project with 2 esp modules with motor attached to blinds, and a nRF52840 as remote

## Structure

`driver` - Contains the code for the ESP modules that control the motors attached to the blinds.
`remote` - Contains the code for the nRF52840 remote that communicates with the ESP modules to control the blinds.
`protocol` - The radio frame format and commands, shared by `driver` and `remote`.
`elf2uf2` - Converts the remote firmware to a UF2 file for drag and drop flashing.
`case` - Contains the 3D model files for the case that houses the nRF52840 remote and ESP modules.

`driver` and `remote` are separate Cargo workspaces, because each builds for its own chip.
Their `.cargo/config.toml` sets the target, and all crates share the `target/` directory at the root.
The root workspace holds `protocol` and `elf2uf2`.

## Building

Run Cargo inside the crate's directory:

```sh
cd driver   # or remote
cargo build --release   # firmware for the chip
cargo run --release     # flash and run through probe-rs
cargo test-host         # unit tests, on this computer
cargo clippy-host       # lint the unit tests
```

`cargo test` at the root tests `protocol` and `elf2uf2`.

### Driver settings

The driver reads its settings from environment variables at build time.
The defaults live in `driver/.cargo/config.toml`, and a variable set in the shell wins:

```sh
BLIND_ID=2 cargo run --release
```

| Variable                         | Meaning                                    |
| -------------------------------- | ------------------------------------------ |
| `BLIND_ID`                       | `1` or `2`, picks the 802.15.4 address     |
| `WIFI_SSID`, `WIFI_PASSWORD`     | Wi-Fi network, for the `mqtt` feature      |
| `MQTT_BROKER`                    | IPv4 address of the MQTT broker            |
| `MQTT_USERNAME`, `MQTT_PASSWORD` | Broker login, left out when empty          |

esp-radio can't run Wi-Fi and IEEE 802.15.4 at the same time yet, so the driver has two
mutually exclusive features: `radio` (default, listens to the remote) and `mqtt` (Home Assistant).

```sh
cargo build --release --no-default-features --features mqtt
```

### Flashing the remote over USB

The Pro Micro nRF52840 has a UF2 bootloader: it shows up as a USB drive, and copying a `.uf2`
file onto it flashes the board. `remote/memory.x` places the firmware after the bootloader at
`0x26000`. From `remote/`:

```sh
cargo uf2
```

This builds the release firmware and converts it with the `elf2uf2` tool in this repository to
`target/thumbv7em-none-eabihf/release/remote.uf2`. Double tap reset to get the bootloader drive,
then drag the file onto it. If the drive is already mounted when you run `cargo uf2`, the file is
copied onto it right away.

Take the CR2032 out first: the board charges `B+` while USB is connected.
