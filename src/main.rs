//! Terminal Pong: you (left paddle) versus a bot (right paddle).
//!
//! Controls: Up/Down move your paddle. Right arrow quits with success,
//! Left arrow quits with a (generic) error.

mod game;

use std::io::{self, Write};
use std::process;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use crossterm::{cursor, execute, terminal};

use game::{covers, Game, BOT_COL, HEIGHT, PLAYER_COL, WIDTH};

const TICK: Duration = Duration::from_millis(60);

fn main() {
    if let Err(e) = run() {
        let _ = teardown();
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    setup()?;
    let mut game = Game::new();
    let mut last = Instant::now();

    loop {
        // Wait for input, but never longer than the time left in this tick.
        let timeout = TICK.saturating_sub(last.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => game.move_player(-1),
                    KeyCode::Down => game.move_player(1),
                    KeyCode::Right => quit(0),
                    KeyCode::Left => quit(1),
                    _ => {}
                }
            }
        }

        if last.elapsed() >= TICK {
            game.step();
            render(&game)?;
            last = Instant::now();
        }
    }
}

/// Restore the terminal and exit with `code`.
fn quit(code: i32) -> ! {
    let _ = teardown();
    process::exit(code);
}

fn setup() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)
}

fn teardown() -> io::Result<()> {
    execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()
}

fn render(g: &Game) -> io::Result<()> {
    let mut out = String::new();
    out.push_str(&format!("You {}    Bot {}\r\n", g.player_score, g.bot_score));

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let cell = if x == PLAYER_COL && covers(g.player_y, y) {
                '#'
            } else if x == BOT_COL && covers(g.bot_y, y) {
                '#'
            } else if x == g.ball_x && y == g.ball_y {
                'O'
            } else {
                ' '
            };
            out.push(cell);
        }
        out.push_str("\r\n");
    }
    out.push_str("Up/Down: move   Right: quit ok   Left: quit error\r\n");

    let mut stdout = io::stdout();
    execute!(stdout, cursor::MoveTo(0, 0))?;
    write!(stdout, "{out}")?;
    stdout.flush()
}
