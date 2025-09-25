use allocative::Allocative;
use std::cmp::Ordering;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Allocative)]
pub struct Vec2 {
    pub x: u32,
    pub y: u32,
}

impl Vec2 {
    pub fn zero() -> Self {
        Vec2 { x: 0, y: 0 }
    }

    pub fn from(x: u32, y: u32) -> Self {
        Vec2 { x, y }
    }

    pub fn add(self, other: u32) -> Self {
        Vec2 {
            x: self.x + other,
            y: self.y + other,
        }
    }

    pub fn sub(self, other: u32) -> Self {
        Vec2 {
            x: self.x - other,
            y: self.y - other,
        }
    }

    pub fn sub2(self, x: u32, y: u32) -> Self {
        Vec2 {
            x: self.x - x,
            y: self.y - y,
        }
    }
}

impl From<(u32, u32)> for Vec2 {
    fn from((x, y): (u32, u32)) -> Self {
        Vec2 { x, y }
    }
}

impl PartialOrd for Vec2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let x = other.x.partial_cmp(&self.x)?;
        let y = other.y.partial_cmp(&self.y)?;
        if x == y {
            Some(x)
        } else {
            None
        }
    }
}

pub struct Pool {
    size: u8,
    max_size: u8,
    sender: mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
    receiver: Arc<Mutex<mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>>>,
}

impl Pool {
    pub fn new(size: u8) -> Pool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        Pool { size: 0, max_size: size, sender, receiver }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&mut self, f: F) {
        if self.size == 0 || self.size <= self.max_size && self.receiver.try_lock().is_ok() {
            let receiver = self.receiver.clone();
            self.size += 1;
            thread::spawn(move || {
                loop {
                    receiver.lock().unwrap().recv().unwrap()();
                }
            });
        }
        self.sender.send(Box::new(f)).unwrap();
    }
}