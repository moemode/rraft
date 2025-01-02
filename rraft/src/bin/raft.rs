// implement two roles a responder which echoes messages
// and a emitter which every second sends a message to the responder
// when the responder receives a message with the data "emit" it will
// itself become an emitter for two seconds

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Duration;
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
struct RequestMessage {
    id: u32,
    data: String,
}

#[derive(Debug)]
struct ResponseMessage {
    id: u32,
    data: String,
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
}

trait HasTerm {
    fn term(&self) -> usize;
}

impl HasTerm for Message {
    fn term(&self) -> usize {
        match self {
            Message::AppendEntryRequest { term, .. } => *term,
            Message::AppendEntryResponse { term, .. } => *term,
        }
    }
}

#[derive(Debug)]
struct TimedMessage {
    delivery_time: Instant,
    from: u32,
    to: u32,
    message: Message,
}

impl PartialEq for TimedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.delivery_time == other.delivery_time
    }
}

impl Eq for TimedMessage {}

impl PartialOrd for TimedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (earliest messages first)
        other.delivery_time.cmp(&self.delivery_time)
    }
}

trait Role {
    fn transition(&mut self, msg: &Message, at: Instant, s: &mut State) -> Option<Box<dyn Role>>;
    fn handle(&mut self, msg: Message, at: Instant, s: &mut State) -> Vec<Message>;
    fn tick(&mut self, at: Instant, s: &mut State) -> Option<Box<dyn Role>>;
    fn tick_msg(&mut self, at: Instant, s: &mut State) -> Vec<Message>;
    fn id(&self) -> u32;
}

struct Candidate {
    votes_received: usize,
    election_started: Instant,
}

impl Role for Candidate {
    fn transition(&mut self, msg: &Message, at: Instant, s: &mut State) -> Option<Box<dyn Role>> {
        let term = msg.term();
        if term > s.current_term {
            s.current_term = term;
            return Some(Box::new(Follower {}));
        }

        None
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

fn main() {
    let mut message_queue: BinaryHeap<TimedMessage> = BinaryHeap::new();
    let start_time = Instant::now();
    let mut current_time = start_time;

    // Create machines: emitter sends pings, responder echoes
    let mut emitter = Machine::new(Box::new(Emitter::new(1, current_time, None)));
    let mut responder = Machine::new(Box::new(Responder::new(2)));

    // Simulation parameters
    let sim_duration = Duration::from_secs(30);
    let tick_interval = Duration::from_millis(10);
    let network_latency = Duration::from_millis(50); // Simulated network delay

    while current_time - start_time < sim_duration {
        // Process messages that have arrived
        while let Some(timed_msg) = message_queue.peek() {
            if timed_msg.delivery_time > current_time {
                break;
            }

            let msg = message_queue.pop().unwrap();
            println!(
                "Time {:?} - Delivering message from {} to {}: {:?}",
                current_time - start_time,
                msg.from,
                msg.to,
                msg.message
            );

            let responses = if msg.to == emitter.id() {
                emitter.handle(msg.message, current_time)
            } else {
                responder.handle(msg.message, current_time)
            };

            // Queue responses with network latency
            for response in responses {
                message_queue.push(TimedMessage {
                    delivery_time: current_time + network_latency,
                    from: msg.to,
                    to: msg.from,
                    message: response,
                });
            }
        }

        // Tick both machines
        for msg in emitter.tick(current_time) {
            println!(
                "Time {:?} - Emitter generated message: {:?}",
                current_time - start_time,
                msg
            );
            message_queue.push(TimedMessage {
                delivery_time: current_time + network_latency,
                from: emitter.id(),
                to: responder.id(),
                message: msg,
            });
        }

        for msg in responder.tick(current_time) {
            println!(
                "Time {:?} - Responder generated message: {:?}",
                current_time - start_time,
                msg
            );
            message_queue.push(TimedMessage {
                delivery_time: current_time + network_latency,
                from: responder.id(),
                to: emitter.id(),
                message: msg,
            });
        }
        // Sleep until next tick
        current_time += tick_interval;
        // Optional: print queue status every second
        /*         if (current_time - start_time).as_secs() % 1 == 0 {
            println!("Queue size: {}", message_queue.len());
        } */
    }
}
