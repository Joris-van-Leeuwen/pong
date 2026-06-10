# Pong

A tiny terminal Pong game in Rust. You control the right paddle; a bot controls
the left paddle and chases the ball.

```
Bot 0    You 1

 #                                    O                     #
 #                                                          #
 #                                                          #
 #                                                          #
```

## Controls

| Key           | Action                          |
| ------------- | ------------------------------- |
| Up            | Move your paddle up             |
| Down          | Move your paddle down           |
| Right / `q`   | Quit with **success** (exit 0)  |
| Left / Escape | Quit with an **error** (exit 1) |

## Run

```sh
cargo run
```

## Configuration

The field, paddles, ball speed, and bot difficulty are all set from the command
line. Pass flags after `--` when using `cargo run`, e.g.:

```sh
cargo run -- --width 80 --height 30 --paddle 6 --ball-speed 60 --bot-delay 800
```

| Flag           | Default | Meaning                                                                                  |
| -------------- | ------- | ---------------------------------------------------------------------------------------- |
| `--width`      | `60`    | Field width in columns.                                                                  |
| `--height`     | `24`    | Field height in rows.                                                                     |
| `--paddle`     | `4`     | Paddle height in rows (both paddles share this size).                                     |
| `--ball-speed` | `90`    | Milliseconds the ball takes to advance one cell — **lower is faster**.                    |
| `--bot-delay`  | `500`   | Milliseconds the bot must hold its direction before reversing — **higher is easier**.    |

Run `cargo run -- --help` to see this list in the terminal.

Validation is enforced at startup (exits with code `2` on bad input): the width
must be at least `5`, the paddle at least `1` and no taller than the field, the
ball speed at least `1`, and the bot delay non-negative.

## Test

```sh
cargo test
```

## Layout

- `src/game.rs` — pure game logic (ball, paddles, collisions, scoring, bot),
  with no terminal I/O so it can be unit-tested directly.
- `src/main.rs` — the terminal loop: reads keys, ticks the game, and renders.

The bot (left paddle) chases the ball one row per tick, but must hold its
current vertical direction for `--bot-delay` milliseconds (500 ms by default)
before it may reverse. That delay makes it overshoot the ball, so it is
beatable — raise it to make the bot easier.
