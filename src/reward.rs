#[derive(Clone, Debug, Default)]
pub struct RewardChannel {
    pub signal_type: Option<u8>,
    pub strength: u8,
}

const REWARD_TICKS: u8 = 60;

impl RewardChannel {
    pub fn trigger(&mut self, signal_type: u8) {
        self.signal_type = Some(signal_type % 16);
        self.strength = REWARD_TICKS;
    }

    pub fn take(&mut self) -> Option<u8> {
        let reward = self.signal_type;
        if self.strength > 0 {
            self.strength -= 1;
        }
        if self.strength == 0 {
            self.signal_type = None;
        }
        reward
    }

    pub fn progress(&self) -> (Option<u8>, f64) {
        let pct = if self.strength > 0 {
            self.strength as f64 / REWARD_TICKS as f64
        } else {
            0.0
        };
        (self.signal_type, pct)
    }
}
