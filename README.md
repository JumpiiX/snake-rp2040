<h1 align="center">Snake&nbsp;·&nbsp;RP2040</h1>

<p align="center"><b>Snake on an 8×8 LED matrix. No OS, no heap, no standard library.</b><br/>
The same game written three times — blocking, interrupt-driven, and async — to see what each style costs.</p>

<p align="center">
  <img src="https://img.shields.io/badge/-%23ce422b-ce422b?style=flat&label=rust&labelColor=1a1a1a" alt="Rust" />
  <img src="https://img.shields.io/badge/target-thumbv6m--none--eabi-1a1a1a?style=flat&labelColor=ce422b" alt="Target" />
  <img src="https://img.shields.io/badge/chip-RP2040-1a1a1a?style=flat&logo=raspberrypi&labelColor=0d0d0d" alt="RP2040" />
  <img src="https://img.shields.io/badge/no__std-yes-1a1a1a?style=flat&labelColor=0d0d0d" alt="no_std" />
  <img src="https://img.shields.io/badge/heap-none-1a1a1a?style=flat&labelColor=0d0d0d" alt="No heap" />
</p>

https://github.com/user-attachments/assets/e6a0edcf-be2a-4da7-845a-e7f05a29112b

---

## Three ways to run a game loop

| Binary | Style | What it does |
|--------|-------|--------------|
| `snake_blocking` | blocking loop | Polls the joystick, sleeps out the tick, advances. Simple to read, spins the CPU through every delay. |
| `snake_interrupt` | timer interrupt | Hardware alarm fires the tick. The handler polls, advances, raises a flag; `main` draws and sleeps in `wfi`. |
| `snake_async` | Embassy | Input, tick and render as separate tasks. On the `async` branch — its HAL conflicts with `rp-pico`. |

Same rules, same renderer, three execution models.

## Running it

A Raspberry Pi Pico on a [ZHAW Lumatrix](https://lumatrix.fun) board, plus a debug probe over SWD.

```sh
rustup target add thumbv6m-none-eabi
cargo install flip-link
curl -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

cargo run --bin snake_blocking
```

Click the joystick to start. Steer with it. Walls wrap, so the only way to die is to bite yourself.

## Layout

| Path | What lives here |
|------|-----------------|
| `src/game/` | Rules — snake, movement, collision, food placement. No hardware types, unit-tested on the host. |
| `src/render.rs` | Game state → LED colours. The only module that knows about the board. |
| `src/input.rs` | Joystick → direction. |
| `src/board/` | Driver for the matrix, joystick and switch. |
| `src/bin/` | One binary per concurrency style. |

The rules know nothing about the hardware, so they run under `cargo test` on a laptop:

```sh
cargo test --target aarch64-apple-darwin
```

## Notes

The snake body is a `heapless::Deque<Position, 64>` — 64 cells is the whole board, so growing can never overflow. Food placement walks forward from a random cell to the next free one, because rejection sampling stalls once the board fills up.

Brightness is capped at 40 of 255. Every white LED pulls roughly 180 mA across its three channels; a full-white 8×8 matrix would ask for far more than USB will give.
