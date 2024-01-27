pub trait Temporal {
    fn tick(&mut self);
}

#[derive(Debug)]
pub struct RaftTimer {
    pub default_timeout: usize,
    pub ticks_left: usize,
}

impl Temporal for RaftTimer {
    fn tick(&mut self) {
        self.ticks_left = self
            .ticks_left
            .checked_sub(1)
            .unwrap_or(self.default_timeout);
    }
}

impl From<usize> for RaftTimer {
    fn from(timeout: usize) -> Self {
        Self {
            default_timeout: timeout,
            ticks_left: timeout,
        }
    }
}
