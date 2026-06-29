# Sigma EFI

Rust engine control firmware for the [microRusEFI](https://www.shop.rusefi.com/shop/p/microrusefi-assembled-ecu-development-module) ECU, built on [Embassy](https://embassy.dev/) and conceptually ported from [rusEFI](https://github.com/rusefi/rusefi).

Licensed under **MIT OR Apache-2.0** (see `LICENSE-MIT` and `LICENSE-APACHE`). rusEFI itself is GPL; this project reimplements algorithms and maps hardware from public board documentation — it does not incorporate rusEFI source code.

## Target engine

**Rotax V990** — locked-in best defaults for a microRusEFI swap (`Profile::best()`).

| Parameter | Best default | Rationale |
|-----------|--------------|-----------|
| Cylinders | 2 (front = 0, rear = 1) | 60° V-twin |
| Displacement | 998 cc | Factory spec |
| Firing | 300° / 420° uneven | OEM crank timing |
| Crank trigger | 6 pulses/rev, 60° apart, VR | Aprilia OEM pattern, MRE pin 45 |
| Cam trigger | Hall, required | Sequential phase sync, MRE pin 25 |
| Spark | 1 plug + 1 coil per cyl | Post-2004 Aprilia / Spyder layout |
| Injection | Sequential (simul. cranking) | OEM EFI behavior |
| Idle | 1,350 RPM | Stable warm idle for 998 cc twin |
| Rev limit | 10,200 soft / 10,500 hard | Aprilia sport redline |

Profile: `crates/core/src/engines/rotax_v990.rs` · Wiring: `crates/board-mre/src/defaults.rs` (`wiring`).

## Hardware

| Item | Value |
|------|-------|
| Board | microRusEFI (rev 0.6.x wiring) |
| MCU | STM32F767VI (verify against your BOM) |
| Smart driver | TLE8888 (injectors, low-side outputs) |
| ETB driver | TLE9201 |
| rusEFI board profile | `meta-info-mre_f7.env` |

Pin assignments live in `crates/board-mre/src/pins.rs`, derived from rusEFI's `board_configuration.cpp` and `connectors/main.yaml`.

## Workspace layout

```
efi/
├── crates/
│   ├── core/          # no_std engine types, fuel/trigger stubs
│   ├── board-mre/     # microRusEFI pins, ADC map, defaults
│   └── firmware/      # Embassy binary (STM32F767)
├── rust-toolchain.toml
└── .cargo/config.toml
```

## Build

Install the embedded target once:

```bash
rustup target add thumbv7em-none-eabihf
```

Build firmware:

```bash
cd efi
cargo build -p sigma-efi-firmware --release --target thumbv7em-none-eabihf
```

Flash (requires [probe-rs](https://probe.rs/) and a SWD probe):

```bash
probe-rs run --chip STM32F767VI target/thumbv7em-none-eabihf/release/sigma-efi-firmware
```

Run host unit tests:

```bash
cargo test -p sigma-efi-core -p sigma-efi-board-mre
```

## Current status

Bring-up firmware only:

- Blinks the **running** LED (PE4) and **comms** LED (PE2)
- Logs board identity via defmt-rtt
- Core crate holds configuration types and placeholder fuel math
- Task modules stubbed (`firmware/src/tasks/`)

## Roadmap (rusEFI parity)

Priority order for porting rusEFI subsystems into Rust:

1. **Trigger** — crank/cam decoding on PC6 / PA5, tooth scheduler
2. **TLE8888** — SPI1 driver for injectors and low-side outputs
3. **Fuel** — speed-density, VE tables, sequential scheduling
4. **Ignition** — coil charge/fire on PD1–PD4
5. **Sensors** — ADC scan (CLT, IAT, MAP, VBatt), NTC math
6. **CAN / USB** — tuning interface (rusEFI protocol or new)
7. **ETB** — TLE9201 drive-by-wire
8. **Storage** — onboard SD (SPI2) for logs and tune persistence

## References

- [microRusEFI shop listing](https://www.shop.rusefi.com/shop/p/microrusefi-assembled-ecu-development-module)
- [rusEFI repository](https://github.com/rusefi/rusefi)
- [microRusEFI hardware repo](https://github.com/rusefi/hw_microRusEfi)
- [rusEFI microRusEFI wiring wiki](https://wiki.rusefi.com/Hardware-microRusEfi-wiring)
- [Embassy STM32F767 docs](https://docs.embassy.dev/embassy-stm32/git/stm32f767vi/index.html)
- [Rotax V990 overview](https://de.zxc.wiki/wiki/Rotax_V990)
- [Aprilia / Rotax crank trigger pattern](https://www.island-underground.com/aprilia/aprilia-fuel-injection/ecu-hardware/ecu-inputs/crankcam-position)
