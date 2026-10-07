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

### Driver settings

The driver reads its settings from environment variables at build time.
The defaults live in `.cargo/config.toml`, and a variable set in the shell wins:

```sh
BLIND_ID=2 cargo run-driver --release
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
cargo build-driver --release --no-default-features --features mqtt
```

### Flashing the remote over USB

The Pro Micro nRF52840 has a UF2 bootloader, and `remote/memory.x` places the firmware after it at
`0x26000`. Without a debug probe, convert the ELF with
[`uf2conv.py`](https://github.com/microsoft/uf2/blob/master/utils/uf2conv.py) and copy it to the
drive that appears after double tapping reset:

```sh
uf2conv.py --family 0xADA52840 --convert --output remote.uf2 target/thumbv7em-none-eabihf/release/remote
```

Take the CR2032 out first: the board charges `B+` while USB is connected.
