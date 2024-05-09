pub enum Role {
    Follower = 0,
    Candidate = 1,
    Leader = 2,
}

pub struct State {
    pub role: Role,
    pub current_term: u64,
    pub voted_for: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            role: Role::Follower,
            current_term: 0,
            voted_for: None,
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            role: Role::Follower,
            current_term: 0,
            voted_for: None,
        }
    }

    pub fn update_term(&mut self, term: u64) {
        self.current_term = term;
    }

    pub fn update_voted_for(&mut self, voted_for: Option<String>) {
        self.voted_for = voted_for;
    }
}