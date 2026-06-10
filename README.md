# Pong

A tiny terminal Pong game in Rust. You control the left paddle; a bot controls
the right paddle and chases the ball.

```
You 1    Bot 0

 #                                    O   #
 #                                        #
 #                                        #
 #                                        #
```

## Controls

| Key     | Action                          |
| ------- | ------------------------------- |
| Up      | Move your paddle up             |
| Down    | Move your paddle down           |
| Right   | Quit with **success** (exit 0)  |
| Left    | Quit with an **error** (exit 1) |

## Run

```sh
cargo run
```

## Test

```sh
cargo test
```

## Layout

- `src/game.rs` — pure game logic (ball, paddles, collisions, scoring, bot),
  with no terminal I/O so it can be unit-tested directly.
- `src/main.rs` — the terminal loop: reads keys, ticks the game, and renders.

The bot moves one row per tick toward the ball, so it is beatable.
