//! Pure Pong game logic — no terminal I/O, so it can be unit-tested directly.

/// Tunable game parameters, supplied at startup (see CLI flags in `main`).
#[derive(Clone, Copy)]
pub struct Config {
    /// Field width in columns.
    pub width: i32,
    /// Field height in rows.
    pub height: i32,
    /// Paddle height in rows (same for both paddles).
    pub paddle_h: i32,
    /// How long the bot must hold its direction before reversing (milliseconds).
    pub bot_delay_ms: i32,
}

pub const DEFAULT_WIDTH: i32 = 60;
pub const DEFAULT_HEIGHT: i32 = 24;
pub const DEFAULT_PADDLE_H: i32 = 4;
pub const DEFAULT_BOT_DELAY_MS: i32 = 500;

impl Default for Config {
    fn default() -> Self {
        Config {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            paddle_h: DEFAULT_PADDLE_H,
            bot_delay_ms: DEFAULT_BOT_DELAY_MS,
        }
    }
}

pub struct Game {
    cfg: Config,
    /// Top row of each paddle.
    pub player_y: i32,
    pub bot_y: i32,
    pub ball_x: i32,
    pub ball_y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub player_score: u32,
    pub bot_score: u32,
    /// Direction the bot's paddle is currently moving: -1 up, 0 still, 1 down.
    pub bot_dir: i32,
    /// Time left before the bot may reverse direction (milliseconds).
    invert_wait_ms: i32,
}

impl Game {
    pub fn new(cfg: Config) -> Self {
        let mid_paddle = cfg.height / 2 - cfg.paddle_h / 2;
        Game {
            cfg,
            player_y: mid_paddle,
            bot_y: mid_paddle,
            ball_x: cfg.width / 2,
            ball_y: cfg.height / 2,
            vel_x: -1,
            vel_y: 1,
            player_score: 0,
            bot_score: 0,
            bot_dir: 0,
            invert_wait_ms: cfg.bot_delay_ms,
        }
    }

    pub fn width(&self) -> i32 {
        self.cfg.width
    }

    pub fn height(&self) -> i32 {
        self.cfg.height
    }

    /// Column the bot's paddle lives on (left edge).
    pub fn bot_col(&self) -> i32 {
        1
    }

    /// Column the player's paddle lives on (right edge).
    pub fn player_col(&self) -> i32 {
        self.cfg.width - 2
    }

    /// True if a paddle whose top is `top` occupies row `y`.
    pub fn covers(&self, top: i32, y: i32) -> bool {
        y >= top && y < top + self.cfg.paddle_h
    }

    /// Move the player's paddle by `dy` rows, clamped to the field.
    pub fn move_player(&mut self, dy: i32) {
        self.player_y = (self.player_y + dy).clamp(0, self.cfg.height - self.cfg.paddle_h);
    }

    /// Advance the game by one tick of `dt_ms` milliseconds: ball, collisions,
    /// scoring, then the bot.
    pub fn step(&mut self, dt_ms: i32) {
        self.ball_x += self.vel_x;
        self.ball_y += self.vel_y;

        // Bounce off the top and bottom walls.
        if self.ball_y <= 0 {
            self.ball_y = 0;
            self.vel_y = self.vel_y.abs();
        } else if self.ball_y >= self.cfg.height - 1 {
            self.ball_y = self.cfg.height - 1;
            self.vel_y = -self.vel_y.abs();
        }

        // Bounce off the paddles: reflect both the velocity and the position so
        // the ball sits in front of the paddle instead of hiding inside it.
        if self.ball_x == self.bot_col() && self.covers(self.bot_y, self.ball_y) {
            self.vel_x = 1;
            self.ball_x = self.bot_col() + 1;
        } else if self.ball_x == self.player_col() && self.covers(self.player_y, self.ball_y) {
            self.vel_x = -1;
            self.ball_x = self.player_col() - 1;
        }

        // Score when the ball leaves the field, then serve toward whoever conceded.
        if self.ball_x <= 0 {
            self.player_score += 1; // past the bot (left)
            self.serve(-1);
        } else if self.ball_x >= self.cfg.width - 1 {
            self.bot_score += 1; // past the player (right)
            self.serve(1);
        }

        self.move_bot(dt_ms);
    }

    /// Reset the ball to the centre, heading horizontally in `dir`.
    fn serve(&mut self, dir: i32) {
        self.ball_x = self.cfg.width / 2;
        self.ball_y = self.cfg.height / 2;
        self.vel_x = dir;
        self.vel_y = 1;
    }

    /// Bot chases the ball one row at a time, but can only reverse direction
    /// after holding the current one for `cfg.bot_delay_ms` — so it overshoots
    /// and is beatable.
    fn move_bot(&mut self, dt_ms: i32) {
        let center = self.bot_y + self.cfg.paddle_h / 2;
        let desired = (self.ball_y - center).signum();

        if self.bot_dir != 0 && desired == -self.bot_dir {
            // The bot wants to reverse: keep going the old way until the timer runs out.
            self.invert_wait_ms -= dt_ms;
            if self.invert_wait_ms <= 0 {
                self.bot_dir = desired;
                self.invert_wait_ms = self.cfg.bot_delay_ms;
            }
        } else {
            // Following the ball (or aligned): no reversal pending, keep the timer charged.
            self.bot_dir = desired;
            self.invert_wait_ms = self.cfg.bot_delay_ms;
        }

        self.bot_y = (self.bot_y + self.bot_dir).clamp(0, self.cfg.height - self.cfg.paddle_h);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A representative tick length for stepping the game in tests.
    const TICK_MS: i32 = 90;

    fn game() -> Game {
        Game::new(Config::default())
    }

    #[test]
    fn ball_moves_each_step() {
        let mut g = game();
        let (x, y) = (g.ball_x, g.ball_y);
        g.step(TICK_MS);
        assert_eq!((g.ball_x, g.ball_y), (x + g.vel_x, y + g.vel_y));
    }

    #[test]
    fn bounces_off_top_wall() {
        let mut g = game();
        g.ball_y = 0;
        g.vel_y = -1;
        g.step(TICK_MS);
        assert!(g.vel_y > 0, "vertical velocity should flip downward");
    }

    #[test]
    fn player_paddle_bounces_ball() {
        let mut g = game();
        g.ball_x = g.player_col() - 1;
        g.vel_x = 1;
        g.player_y = g.ball_y - 1; // ensure the paddle covers the ball row
        g.step(TICK_MS);
        assert_eq!(g.vel_x, -1, "ball should rebound toward the bot");
        assert_eq!(
            g.ball_x,
            g.player_col() - 1,
            "ball should sit in front of the paddle, not inside it"
        );
    }

    #[test]
    fn missed_ball_scores_for_bot() {
        let mut g = game();
        g.ball_x = g.width() - 2;
        g.vel_x = 1;
        g.player_y = 0; // keep the paddle clear of the ball
        g.ball_y = g.height() - 1;
        g.step(TICK_MS);
        assert_eq!(g.bot_score, 1);
        assert_eq!(g.ball_x, g.width() / 2, "ball returns to centre after a point");
    }

    #[test]
    fn player_cannot_leave_the_field() {
        let mut g = game();
        g.move_player(-1000);
        assert_eq!(g.player_y, 0);
        g.move_player(1000);
        assert_eq!(g.player_y, g.height() - DEFAULT_PADDLE_H);
    }

    #[test]
    fn bot_tracks_the_ball() {
        let mut g = game();
        g.bot_y = 0;
        g.ball_y = g.height() - 1;
        let before = g.bot_y;
        g.step(TICK_MS);
        assert!(g.bot_y > before, "bot should move toward the ball");
    }

    #[test]
    fn bot_waits_before_reversing() {
        let mut g = game();
        g.bot_y = 8;
        // Ball below: bot commits to moving down.
        g.ball_y = g.height() - 1;
        g.step(100);
        assert_eq!(g.bot_dir, 1);
        let y_when_reversal_requested = g.bot_y;

        // Ball jumps above: the bot wants to reverse but must wait 500ms.
        g.ball_y = 0;
        g.step(200);
        g.step(200); // 400ms elapsed — still under the limit
        assert_eq!(g.bot_dir, 1, "bot holds its old direction during the delay");
        assert!(
            g.bot_y >= y_when_reversal_requested,
            "bot keeps moving the old way (overshoots) while waiting"
        );

        // Crossing 500ms total finally lets it reverse.
        g.step(200);
        assert_eq!(g.bot_dir, -1, "bot reverses once the delay has elapsed");
    }

    #[test]
    fn config_is_respected() {
        let g = Game::new(Config {
            width: 30,
            height: 16,
            paddle_h: 6,
            bot_delay_ms: 200,
        });
        assert_eq!(g.width(), 30);
        assert_eq!(g.height(), 16);
        assert_eq!(g.player_col(), 28);
        assert!(g.covers(0, 5) && !g.covers(0, 6), "paddle spans its height");
    }
}
