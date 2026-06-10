//! Pure Pong game logic — no terminal I/O, so it can be unit-tested directly.

pub const WIDTH: i32 = 60;
pub const HEIGHT: i32 = 24;
pub const PADDLE_H: i32 = 4;

/// Column the bot's paddle lives on (left edge).
pub const BOT_COL: i32 = 1;
/// Column the player's paddle lives on (right edge).
pub const PLAYER_COL: i32 = WIDTH - 2;

/// True if a paddle whose top is `top` occupies row `y`.
pub fn covers(top: i32, y: i32) -> bool {
    y >= top && y < top + PADDLE_H
}

pub struct Game {
    /// Top row of each paddle.
    pub player_y: i32,
    pub bot_y: i32,
    pub ball_x: i32,
    pub ball_y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub player_score: u32,
    pub bot_score: u32,
}

impl Game {
    pub fn new() -> Self {
        let mid_paddle = HEIGHT / 2 - PADDLE_H / 2;
        Game {
            player_y: mid_paddle,
            bot_y: mid_paddle,
            ball_x: WIDTH / 2,
            ball_y: HEIGHT / 2,
            vel_x: -1,
            vel_y: 1,
            player_score: 0,
            bot_score: 0,
        }
    }

    /// Move the player's paddle by `dy` rows, clamped to the field.
    pub fn move_player(&mut self, dy: i32) {
        self.player_y = (self.player_y + dy).clamp(0, HEIGHT - PADDLE_H);
    }

    /// Advance the game by one tick: ball, collisions, scoring, then bot.
    pub fn step(&mut self) {
        self.ball_x += self.vel_x;
        self.ball_y += self.vel_y;

        // Bounce off the top and bottom walls.
        if self.ball_y <= 0 {
            self.ball_y = 0;
            self.vel_y = self.vel_y.abs();
        } else if self.ball_y >= HEIGHT - 1 {
            self.ball_y = HEIGHT - 1;
            self.vel_y = -self.vel_y.abs();
        }

        // Bounce off the paddles.
        if self.ball_x == BOT_COL && covers(self.bot_y, self.ball_y) {
            self.vel_x = 1;
        } else if self.ball_x == PLAYER_COL && covers(self.player_y, self.ball_y) {
            self.vel_x = -1;
        }

        // Score when the ball leaves the field, then serve toward whoever conceded.
        if self.ball_x <= 0 {
            self.player_score += 1; // past the bot (left)
            self.serve(-1);
        } else if self.ball_x >= WIDTH - 1 {
            self.bot_score += 1; // past the player (right)
            self.serve(1);
        }

        self.move_bot();
    }

    /// Reset the ball to the centre, heading horizontally in `dir`.
    fn serve(&mut self, dir: i32) {
        self.ball_x = WIDTH / 2;
        self.ball_y = HEIGHT / 2;
        self.vel_x = dir;
        self.vel_y = 1;
    }

    /// Bot chases the ball one row at a time, so it's beatable.
    fn move_bot(&mut self) {
        let center = self.bot_y + PADDLE_H / 2;
        if center < self.ball_y {
            self.bot_y += 1;
        } else if center > self.ball_y {
            self.bot_y -= 1;
        }
        self.bot_y = self.bot_y.clamp(0, HEIGHT - PADDLE_H);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ball_moves_each_step() {
        let mut g = Game::new();
        let (x, y) = (g.ball_x, g.ball_y);
        g.step();
        assert_eq!((g.ball_x, g.ball_y), (x + g.vel_x, y + g.vel_y));
    }

    #[test]
    fn bounces_off_top_wall() {
        let mut g = Game::new();
        g.ball_y = 0;
        g.vel_y = -1;
        g.step();
        assert!(g.vel_y > 0, "vertical velocity should flip downward");
    }

    #[test]
    fn player_paddle_bounces_ball() {
        let mut g = Game::new();
        g.ball_x = PLAYER_COL - 1;
        g.vel_x = 1;
        g.player_y = g.ball_y - 1; // ensure the paddle covers the ball row
        g.step();
        assert_eq!(g.vel_x, -1, "ball should rebound toward the bot");
    }

    #[test]
    fn missed_ball_scores_for_bot() {
        let mut g = Game::new();
        g.ball_x = WIDTH - 2;
        g.vel_x = 1;
        g.player_y = 0; // keep the paddle clear of the ball
        g.ball_y = HEIGHT - 1;
        g.step();
        assert_eq!(g.bot_score, 1);
        assert_eq!(g.ball_x, WIDTH / 2, "ball returns to centre after a point");
    }

    #[test]
    fn player_cannot_leave_the_field() {
        let mut g = Game::new();
        g.move_player(-1000);
        assert_eq!(g.player_y, 0);
        g.move_player(1000);
        assert_eq!(g.player_y, HEIGHT - PADDLE_H);
    }

    #[test]
    fn bot_tracks_the_ball() {
        let mut g = Game::new();
        g.bot_y = 0;
        g.ball_y = HEIGHT - 1;
        let before = g.bot_y;
        g.step();
        assert!(g.bot_y > before, "bot should move toward the ball");
    }
}
