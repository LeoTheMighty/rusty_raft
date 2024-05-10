#[derive(PartialEq, Clone)]
pub enum Role {
    Follower = 0,
    Candidate = 1,
    Leader = 2,
}

impl std::fmt::Debug for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Follower =>  write!(f, "FOLLOWER "),
            Role::Candidate => write!(f, "CANDIDATE"),
            Role::Leader =>    write!(f, "LEADER   "),
        }
    }
}

pub struct State {
    pub role: Role,
    pub current_term: u64,
    pub voted_for: Option<String>,
    pub votes_received: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            role: Role::Follower,
            current_term: 0,
            voted_for: None,
            votes_received: 0,
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            role: Role::Follower,
            current_term: 0,
            voted_for: None,
            votes_received: 0,
        }
    }
}