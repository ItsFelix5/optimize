use crate::buffer::Buffer;
use crate::config::Config;
use crate::gui::View;
use crate::library::Library;
use allocative::Allocative;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::env::current_exe;
use std::path::PathBuf;

mod buffer;
mod config;
mod gui;
mod input;
mod library;
mod window;

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
        Vec2 {x, y}
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

#[derive(Allocative)]
struct State {
    dirty: bool,
    #[allocative(skip)]
    config: Config,
    library: Library,
    buffer: Buffer,
    #[allocative(skip)]
    view: View
}

impl State {
    pub fn new() -> Self {
        State {
            dirty: false,
            config: Config::default(),
            library: Library::new(),
            buffer: Buffer::new(Vec2::zero()),
            view: View::gallery(),
        }
    }

    pub fn update(&mut self) {
        self.dirty = true;
    }
}

thread_local! {
    static GLOBAL_STATE: RefCell<State> = RefCell::new(State::new());
}

pub(crate) fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    GLOBAL_STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        f(&mut s)
    })
}

fn main() {
    let exe = current_exe().unwrap();
    let dir: PathBuf = exe.parent().unwrap().to_path_buf();
    with_state(|state| {
        state.config.load(dir.join("nanogallery.cfg"));
        for lib in state.config.libraries.clone().iter() {
            state.library.load(PathBuf::from(lib));
        }
    });
    window::create();
}
