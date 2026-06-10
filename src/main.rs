//! Terminal Pong: you (right paddle) versus a bot (left paddle).
//!
//! Controls: Up/Down move your paddle. Right arrow or `q` quits with success,
//! Left arrow or Escape quits with a (generic) error.
//!
//! The field size, paddle size, ball speed, and bot delay are all set from the
//! command line — run with `--help` to see the options.

mod game;

use std::io::{self, Write};
use std::process;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use crossterm::{cursor, execute, terminal};

use game::{Config, Game, DEFAULT_BOT_DELAY_MS, DEFAULT_HEIGHT, DEFAULT_PADDLE_H, DEFAULT_WIDTH};

/// Default milliseconds the ball takes to advance one cell (lower = faster).
const DEFAULT_BALL_SPEED_MS: u64 = 90;

/// Terminal Pong against a bot.
#[derive(Parser)]
#[command(about, long_about = None)]
struct Args {
    /// Field width in columns.
    #[arg(long, default_value_t = DEFAULT_WIDTH)]
    width: i32,

    /// Field height in rows.
    #[arg(long, default_value_t = DEFAULT_HEIGHT)]
    height: i32,

    /// Paddle height in rows (applies to both paddles).
    #[arg(long, default_value_t = DEFAULT_PADDLE_H)]
    paddle: i32,

    /// Ball speed as milliseconds per cell moved — lower is faster.
    #[arg(long, default_value_t = DEFAULT_BALL_SPEED_MS)]
    ball_speed: u64,

    /// Bot reversal delay in milliseconds — higher makes the bot easier to beat.
    #[arg(long, default_value_t = DEFAULT_BOT_DELAY_MS)]
    bot_delay: i32,
}

impl Args {
    /// Reject values that would break rendering or the game logic.
    fn validate(&self) -> Result<(), String> {
        if self.paddle < 1 {
            return Err("paddle must be at least 1".into());
        }
        if self.height < self.paddle {
            return Err("height must be at least the paddle height".into());
        }
        // Need distinct paddle columns (1 and width-2) with a gap between them.
        if self.width < 5 {
            return Err("width must be at least 5".into());
        }
        if self.ball_speed < 1 {
            return Err("ball-speed must be at least 1 (ms)".into());
        }
        if self.bot_delay < 0 {
            return Err("bot-delay cannot be negative".into());
        }
        Ok(())
    }
}

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("error: {e}");
        process::exit(2);
    }

    let cfg = Config {
        width: args.width,
        height: args.height,
        paddle_h: args.paddle,
        bot_delay_ms: args.bot_delay,
    };

    if let Err(e) = run(cfg, Duration::from_millis(args.ball_speed)) {
        let _ = teardown();
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run(cfg: Config, tick: Duration) -> io::Result<()> {
    setup()?;
    let mut game = Game::new(cfg);
    let mut last = Instant::now();

    loop {
        // Wait for input, but never longer than the time left in this tick.
        let timeout = tick.saturating_sub(last.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => game.move_player(-1),
                    KeyCode::Down => game.move_player(1),
                    KeyCode::Right | KeyCode::Char('q') => quit(0),
                    KeyCode::Left | KeyCode::Esc => quit(1),
                    _ => {}
                }
            }
        }

        if last.elapsed() >= tick {
            game.step(tick.as_millis() as i32);
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
    out.push_str(&format!("Bot {}    You {}\r\n", g.bot_score, g.player_score));

    for y in 0..g.height() {
        for x in 0..g.width() {
            let cell = if x == g.player_col() && g.covers(g.player_y, y) {
                '#'
            } else if x == g.bot_col() && g.covers(g.bot_y, y) {
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

    let mut stdout = io::stdout();
    execute!(stdout, cursor::MoveTo(0, 0))?;
    write!(stdout, "{out}")?;
    stdout.flush()
}
