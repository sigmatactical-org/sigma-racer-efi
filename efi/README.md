# Sigma EFI

Rust engine control firmware for the [microRusEFI](https://www.shop.rusefi.com/shop/p/microrusefi-assembled-ecu-development-module) ECU, built on [Embassy](https://embassy.dev/) and conceptually ported from [rusEFI](https://github.com/rusefi/rusefi).

Licensed under **MIT OR Apache-2.0** (see `LICENSE-MIT` and `LICENSE-APACHE`). rusEFI itself is GPL; this project reimplements algorithms and maps hardware from public board documentation — it does not incorporate rusEFI source code.

## Architecture

The core is **engine-agnostic**. Board support (microRusEFI pins, ADC scaling) lives separately from engine profiles (cylinder count, firing order, trigger wheel, rev limits).

```
src/
├── config.rs          # EngineConfig, injection/ignition modes, firing presets
├── engines/
│   ├── profile.rs     # EngineProfile (shared struct)
│   ├── yamaha_cp3.rs  # Yamaha CP3 triple
│   └── rotax_v990.rs  # Rotax V990 V-twin
├── defaults.rs        # MRE wiring (maps cylinder index → outputs)
└── pins.rs            # STM32 pin map
```

Pick an engine at **compile time** with Cargo features. Add a new engine by creating `src/engines/your_engine.rs` and a matching feature flag.

### Built-in engine profiles

| Feature | Engine | Cylinders |
|---------|--------|-----------|
| `engine-yamaha-cp3` | Yamaha CP3 (MT-09, XSR900, R9) | 3 |
| `engine-rotax-v990` | Rotax V990 (Aprilia RSV, Spyder) | 2 |

## Hardware

| Item | Value |
|------|-------|
| Board | microRusEFI (rev 0.6.x wiring) |
| MCU | STM32F767VI (verify against your BOM) |
| Smart driver | TLE8888 (injectors, low-side outputs) |
| ETB driver | TLE9201 |
| rusEFI board profile | `meta-info-mre_f7.env` |

Pin assignments live in `src/pins.rs`, derived from rusEFI's `board_configuration.cpp` and `connectors/main.yaml`.

## Build

Install the embedded target once:

```bash
rustup target add thumbv7em-none-eabihf
```

Build firmware (choose one engine feature):

```bash
cd embedded/efi
cargo build --features firmware,engine-yamaha-cp3 --release --target thumbv7em-none-eabihf
# or
cargo build --features firmware,engine-rotax-v990 --release --target thumbv7em-none-eabihf
```

Flash (requires [probe-rs](https://probe.rs/) and a SWD probe):

```bash
probe-rs run --chip STM32F767VI target/thumbv7em-none-eabihf/release/sigma-efi
```

Run host unit tests:

```bash
cargo test
```

## Current status

Bring-up firmware only:

- Blinks the **running** LED (PE4) and **comms** LED (PE2)
- Logs board + selected engine profile via defmt-rtt
- Core crate holds configuration types and placeholder fuel math
- Task modules stubbed (`src/bin/tasks/`)

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
- [Yamaha CP3 engine overview](https://motofomo.com/yamaha-cp3-history-models/)
- [Rotax V990 overview](https://de.zxc.wiki/wiki/Rotax_V990)
