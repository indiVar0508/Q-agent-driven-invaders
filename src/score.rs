use crate::frame::{Drawable, Frame};

#[derive(Default)]
pub struct Score {
    count: u16,
    best_score: u16,
}

impl Score {
    pub fn new() -> Self {
        Self { count: 0, best_score: 0 }
    }

    pub fn add_points(&mut self, amount: u16) {
        self.count += amount;
    }

    pub fn update_best_points(&mut self) {
        if self.count > self.best_score{
            self.best_score += self.count;
        }
    }

    pub fn reset_count(&mut self) {
        self.count = 0;
    }

    pub fn get_count(&mut self) -> u16 {
        self.count
    }

    pub fn get_best_score(&mut self) -> u16 {
        self.best_score
    }

    pub fn write_best_score(&self, frame: &mut Frame) {
        // format our score string
        let formatted = format!("Best SCORE: {:0>4}", self.best_score);

        // iterate over all characters
        for (i, c) in formatted.chars().enumerate() {
            // put them in the first row
            frame[i][1] = c;
        }
    }
}

impl Drawable for Score {
    fn draw(&self, frame: &mut Frame) {
        // format our score string
        let formatted = format!("SCORE: {:0>4}", self.count);

        // iterate over all characters
        for (i, c) in formatted.chars().enumerate() {
            // put them in the first row
            frame[i][0] = c;
        }
    }
}
