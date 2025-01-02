use std::time::Instant;

#[derive(Debug)]
struct LogEntry {}

struct State {
    current_term: usize,
    voted_for: Option<usize>,
    log: Vec<LogEntry>,
    commit_index: usize,
    last_applied: usize,
    n_nodes: usize,
}

#[derive(Debug)]
enum Message {
    AppendEntryRequest {
        term: usize,
        leader_id: usize,
        prev_log_index: usize,
        prev_log_term: usize,
        entries: Vec<LogEntry>,
        leader_commit: usize,
    },

    AppendEntryResponse {
        term: usize,
        success: bool,
        from: usize,
    },

    RequestVoteRequest {
        term: usize,
        candidate_id: usize,
        last_log_index: usize,
        last_log_term: usize,
    },

    RequestVoteResponse {
        term: usize,
        vote_granted: bool,
        from: usize,
    },
}

trait HasTerm {
    fn term(&self) -> usize;
}

impl HasTerm for Message {
    fn term(&self) -> usize {
        match self {
            Message::AppendEntryRequest { term, .. } => *term,
            Message::AppendEntryResponse { term, .. } => *term,
            Message::RequestVoteRequest { term, .. } => *term,
            Message::RequestVoteResponse { term, .. } => *term,
        }
    }
}

trait Role {
    fn transition(&mut self, msg: &Message, at: Instant, s: &mut State) -> Option<Box<dyn Role>>;
    fn handle(&mut self, msg: Message, at: Instant, s: &mut State) -> Vec<Message>;
    fn tick(&mut self, at: Instant, s: &mut State) -> Option<Box<dyn Role>>;
    fn tick_msg(&mut self, at: Instant, s: &mut State) -> Vec<Message>;
}

struct Candidate {
    votes_received: HashSet<usize>,
    election_started: Instant,
}

impl Role for Candidate {
    fn transition(&mut self, msg: &Message, at: Instant, s: &mut State) -> Option<Box<dyn Role>> {
        let term = msg.term();
        if term > s.current_term {
            s.current_term = term;
            return Some(Box::new(Follower {}));
        }
        match msg {
            Message::AppendEntryRequest { term, .. } => {
                if *term >= s.current_term {
                    return Some(Box::new(Follower {}));
                }
                None
            }
            Message::RequestVoteResponse {
                vote_granted, from, ..
            } => {
                if *vote_granted {
                    self.votes_received.insert(*from);
                    if self.votes_received.len() > (s.n_nodes / 2) + 1 {
                        return Some(Box::new(Leader::new()));
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn handle(&mut self, msg: Message, at: Instant, s: &mut State) -> Vec<Message> {
        todo!()
    }

    fn tick(&mut self, at: Instant, s: &mut State) -> Option<Box<dyn Role>> {
        todo!()
    }

    fn tick_msg(&mut self, at: Instant, s: &mut State) -> Vec<Message> {
        todo!()
    }

    fn id(&self) -> u32 {
        todo!()
    }
}

struct Machine {
    role: Box<dyn Role>,
    last_tick: Instant,
    state: State,
}

impl Machine {
    fn new(role: Box<dyn Role>) -> Self {
        Machine {
            role,
            last_tick: Instant::now(),
            state: State {
                current_term: 0,
                voted_for: None,
                log: vec![],
                commit_index: 0,
                last_applied: 0,
                n_nodes: 0,
            },
        }
    }

    fn tick(&mut self, at: Instant) -> Vec<Message> {
        if let Some(new_role) = self.role.tick(at, &mut self.state) {
            self.role = new_role;
        }
        self.last_tick = at;
        self.role.tick_msg(at, &mut self.state)
    }

    fn handle(&mut self, msg: Message, at: Instant) -> Vec<Message> {
        if self.state.current_term > msg.term() {
            return vec![];
        }
        // First check if message triggers a role transition
        if let Some(new_role) = self.role.transition(&msg, at, &mut self.state) {
            self.role = new_role;
        }
        // Then handle the message with current role
        self.role.handle(msg, at, &mut self.state)
    }

    fn id(&self) -> u32 {
        self.role.id()
    }
}
