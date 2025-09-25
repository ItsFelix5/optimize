#![feature(portable_simd)]

use crate::buffer::Buffer;
use crate::config::Config;
use crate::gui::View;
use crate::library::Library;
use crate::util::{Pool, Vec2};
use allocative::Allocative;
use std::env::current_exe;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

mod buffer;
mod config;
mod gui;
mod input;
mod library;
mod window;
mod util;

#[derive(Allocative)]
struct State {
    dirty: bool,
    #[allocative(skip)]
    config: Config,
    library: Library,
    buffer: Buffer,
    #[allocative(skip)]
    view: View,
    #[allocative(skip)]
    pub thread_pool: Pool,
}

impl State {
    pub fn new() -> Self {
        State {
            dirty: false,
            config: Config::default(),
            library: Library::new(),
            buffer: Buffer::new(Vec2::zero()),
            view: View::gallery(),
            thread_pool: Pool::new(6),
        }
    }

    pub fn update(&mut self) {
        self.dirty = true;
    }
}

static GLOBAL_STATE: LazyLock<RwLock<State>> = LazyLock::new(|| RwLock::new(State::new()));

fn main() {
    let exe = current_exe().unwrap();
    let dir: PathBuf = exe.parent().unwrap().to_path_buf();
    let mut state = GLOBAL_STATE.write().unwrap();
    state.config.load(dir.join("nanogallery.cfg"));
    for lib in state.config.libraries.clone().iter() {
        state.library.load(PathBuf::from(lib));
    }
    drop(state);
    window::create();
}
