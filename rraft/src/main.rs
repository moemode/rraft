use std::time::{Duration, Instant};

#[derive(Debug)]
struct HeartbeatMsg;

struct Heartbeater {
    next_heartbeat: Instant,
    interval: Duration,
}

struct Follower {
    convert_to_heartbeater: Instant,
}

struct Raft {
    replica_id: u64,
    state: State,
}

impl Raft {
    fn new(replica_id: u64, created_at: Instant) -> Self {
        Self {
            replica_id,
            state: State::Follower(Follower::new(created_at)),
        }
    }

    fn tick(&mut self, at: Instant) {
        self.state = match self.state {
            State::Heartbeater(ref mut hb) => hb.tick_state(at),
            State::Follower(ref mut f) => f.tick_state(at),
        };
    }

    fn send_msg(&mut self, msg: HeartbeatMsg) {
        match self.state {
            State::Heartbeater(ref mut hb) => {
                // Send heartbeat
            }
            State::Follower(ref mut f) => {
                // Ignore
            }
        }
    }
}

enum State {
    Heartbeater(Heartbeater),
    Follower(Follower),
}

impl Heartbeater {
    fn new(interval: Duration, created_at: Instant) -> Self {
        Self {
            next_heartbeat: created_at,
            interval,
        }
    }

    pub fn tick_msg(&mut self, at: Instant) -> Option<HeartbeatMsg> {
        if at >= self.next_heartbeat {
            self.next_heartbeat = at + self.interval;
            Some(HeartbeatMsg)
        } else {
            None
        }
    }

    pub fn tick_state(self, at: Instant) -> State {
        State::Heartbeater(self)
    }
}

impl Follower {
    fn new(created_at: Instant) -> Self {
        Self {
            convert_to_heartbeater: created_at + Duration::from_secs(5),
        }
    }

    pub fn tick_msg(&mut self, at: Instant) -> Option<HeartbeatMsg> {
        None
    }

    pub fn tick_state(self, at: Instant) -> State {
        if at >= self.convert_to_heartbeater {
            State::Heartbeater(Heartbeater::new(Duration::from_secs(2), at))
        } else {
            State::Follower(self)
        }
    }
}

fn main() {
    let now = Instant::now();
    // Example usage
    let mut heartbeater = Heartbeater::new(Duration::from_secs(2), now);
    // Simulate ticks with loop
    for i in 0..10 {
        if let Some(msg) = heartbeater.tick(now + Duration::from_secs(i)) {
            println!("Sending heartbeat: {:?}", msg);
        }
    }
}
