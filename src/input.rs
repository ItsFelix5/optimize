use crate::GLOBAL_STATE;
use allocative::FlameGraphBuilder;
use minifb::{InputCallback, Key};
use std::str::FromStr;

pub struct Input {}

impl InputCallback for Input {
    fn add_char(&mut self, _uni_char: u32) {}

    fn set_key_state(&mut self, key: Key, down: bool) {
        if !down {
            return;
        }

        let state = &mut *GLOBAL_STATE.write().unwrap();
        match key {
            Key::Up => {
                state.view.scroll -= 100.min(state.view.scroll);
            }
            Key::Down => {
                state.view.scroll = (state.view.scroll + 100).min(state.view.height - state.buffer.size.y);
            }
            Key::Home => {
                state.view.scroll = 0;
            }
            Key::End => {
                state.view.scroll = state.view.height - (state.buffer.size.y).min(state.view.height);
            }
            Key::PageUp => {
                state.view.scroll -= (state.buffer.size.y).min(state.view.scroll);
            }
            Key::PageDown => {
                state.view.scroll = (state.view.scroll + state.buffer.size.y).min(state.view.height - state.buffer.size.y);
            }
            Key::F10 => {
                let mut builder = FlameGraphBuilder::default();
                builder.visit_root(state);
                let output = builder.finish();
                println!("{}", output.warnings());
                let flamegraph = output.flamegraph();
                let total = flamegraph.total_size() as f32 / 1024.0;
                let mut unused = 0.0;
                let mut screen = 0.0;
                let mut image_data = 0.0;
                let mut images = 0.0;
                for line in flamegraph.write().lines() {
                    println!("{}", line);
                    let count = usize::from_str(line.rsplit_once(' ').unwrap_or(("", "0")).1).unwrap_or(0) as f32 / 1024.0;

                    if line.contains("unused") {
                        unused += count;
                    } else if line.starts_with("optimize::State;buffer;optimize::buffer::Buffer;data;alloc::vec::Vec<u32>;ptr;u32") {
                        screen += count;
                    } else if line.starts_with("optimize::State;library;optimize::library::Library;images;alloc::vec::Vec<optimize::library::Image>;ptr;optimize::library::Image;buffer;core::cell::RefCell<core::option::Option<optimize::buffer::Buffer>>;data;core::option::Option<optimize::buffer::Buffer>;Some;optimize::buffer::Buffer;data") {
                        image_data += count;
                    } else if line.starts_with("optimize::State;library") {
                        images += count;
                    }
                }

                println!("total: {total:.4}KiB unused: {unused:.4}KiB");
                println!("screen: {screen:.4}KiB ({}x{}x4={:.4}KiB)", state.buffer.size.x, state.buffer.size.y, (state.buffer.size.x * state.buffer.size.y * 4) as f32 / 1024.0);
                println!("images: {images:.4}KiB data: {image_data:.4}KiB ({}/{})", state.library.images.iter().filter(|i| i.loaded()).count(), state.library.images.len());
            }
            _ => {}
        }
        state.update();
    }
}
