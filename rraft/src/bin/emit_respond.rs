// implement two roles a responder which echoes messages
// and a emitter which every second sends a message to the responder
// when the responder receives a message with the data "emit" it will 
// itself become an emitter for two seconds

use std::time::Instant;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Duration;

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
    Request(RequestMessage),
    Response(ResponseMessage),
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
    fn transition(&mut self, msg: &Message, at: Instant) -> Option<Box<dyn Role>>;
    fn handle(&mut self, msg: Message, at: Instant) -> Vec<Message>;
    fn tick(&mut self, at: Instant) -> Option<Box<dyn Role>>;
    fn tick_msg(&mut self, at: Instant) -> Vec<Message>;
    fn id(&self) -> u32;
}

struct Emitter {
    id: u32,
    counter: u32,
    stop_at: Option<Instant>,
    last_emit: Instant,
}

impl Emitter {
    fn new(id: u32, at: Instant, duration_secs: Option<u64>) -> Self {
        Emitter {
            id,
            counter: 0,
            stop_at: duration_secs.map(|secs| at + std::time::Duration::from_secs(secs)),
            last_emit: at,
        }
    }
}

impl Role for Emitter {
    fn transition(&mut self, _msg: &Message, _at: Instant) -> Option<Box<dyn Role>> {
        None // Emitter doesn't transition based on messages
    }

    fn handle(&mut self, _msg: Message, _at: Instant) -> Vec<Message> {
        vec![] // Emitter doesn't handle incoming messages
    }

    fn tick(&mut self, at: Instant) -> Option<Box<dyn Role>> {
        if let Some(stop_at) = self.stop_at {
            if at >= stop_at {
                return Some(Box::new(Responder::new(self.id)));
            }
        }
        None
    }

    fn tick_msg(&mut self, at: Instant) -> Vec<Message> {
        if at.duration_since(self.last_emit).as_secs() >= 1 {
            self.last_emit = at;
            self.counter += 1;
            vec![Message::Request(RequestMessage {
                id: self.counter,
                data: "ping".to_string(),
            })]
        } else {
            vec![]
        }
    }

    fn id(&self) -> u32 {
        self.id
    }
}

struct Responder {
    id: u32,
}

impl Responder {
    fn new(id: u32) -> Self {
        Responder { id }
    }
}

impl Role for Responder {
    fn transition(&mut self, msg: &Message, at: Instant) -> Option<Box<dyn Role>> {
        match msg {
            Message::Request(req) if req.data == "emit" => {
                Some(Box::new(Emitter::new(self.id, at, Some(2))))
            }
            _ => None,
        }
    }

    fn handle(&mut self, msg: Message, _at: Instant) -> Vec<Message> {
        match msg {
            Message::Request(req) => vec![Message::Response(ResponseMessage {
                id: req.id,
                data: req.data,
            })],
            _ => vec![],
        }
    }

    fn tick(&mut self, _at: Instant) -> Option<Box<dyn Role>> {
        None
    }

    fn tick_msg(&mut self, _at: Instant) -> Vec<Message> {
        vec![]
    }

    fn id(&self) -> u32 {
        self.id
    }
}

struct Machine {
    role: Box<dyn Role>,
    last_tick: Instant,
}

impl Machine {
    fn new(role: Box<dyn Role>) -> Self {
        Machine {
            role,
            last_tick: Instant::now(),
        }
    }

    fn tick(&mut self, at: Instant) -> Vec<Message> {
        if let Some(new_role) = self.role.tick(at) {
            self.role = new_role;
        }
        self.last_tick = at;
        self.role.tick_msg(at)
    }

    fn handle(&mut self, msg: Message, at: Instant) -> Vec<Message> {
        // First check if message triggers a role transition
        if let Some(new_role) = self.role.transition(&msg, at) {
            self.role = new_role;
        }
        // Then handle the message with current role
        self.role.handle(msg, at)
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
            println!("Time {:?} - Delivering message from {} to {}: {:?}", 
                    current_time - start_time, msg.from, msg.to, msg.message);
            
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
            println!("Time {:?} - Emitter generated message: {:?}", 
                    current_time - start_time, msg);
            message_queue.push(TimedMessage {
                delivery_time: current_time + network_latency,
                from: emitter.id(),
                to: responder.id(),
                message: msg,
            });
        }
        
        for msg in responder.tick(current_time) {
            println!("Time {:?} - Responder generated message: {:?}", 
                    current_time - start_time, msg);
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