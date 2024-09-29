// src/pos.rs
use rand::Rng;

pub struct Participant {
    pub address: String,
    pub stake: u64,
}

pub struct PoS {
    pub participants: Vec<Participant>,
}

impl PoS {
    pub fn new() -> Self {
        PoS { participants: Vec::new() }
    }

    pub fn add_participant(&mut self, address: String, stake: u64) {
        self.participants.push(Participant { address, stake });
    }

    pub fn select_validator(&self) -> Option<&Participant> {
        if self.participants.is_empty() {
            return None;
        }

        let total_stake: u64 = self.participants.iter().map(|p| p.stake).sum();
        let mut rng = rand::thread_rng();
        let mut rand_stake = rng.gen_range(0..total_stake);

        for participant in &self.participants {
            if rand_stake < participant.stake {
                return Some(participant);
            }
            rand_stake -= participant.stake;
        }

        None
    }
}
