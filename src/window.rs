use crate::buffer::Buffer;
use crate::gui::View;
use crate::input::Input;
use crate::{Vec2, GLOBAL_STATE};
use minifb::{Window, WindowOptions};

pub fn create() {
    let mut window = Window::new(
        "NanoGallery",
        100,
        100,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
        .unwrap();

    window.set_input_callback(Box::new(Input {}));
    window.set_target_fps(60);

    while window.is_open() {
        let buffer_opt = {
            let state = &mut *GLOBAL_STATE.write().unwrap();
            let (width, height) = window.get_size();
            let width = width.clamp(100, 1920) as u32;
            let height = height.clamp(100, 1080) as u32;
            let size = Vec2::from(width, height);

            if size != state.buffer.size {
                state.buffer = Buffer::new(size);
                View::resize(state);
                state.update();
            }

            window.get_scroll_wheel().map(|(_, vertical)| {
                state.view.scroll = if state.view.height <= state.buffer.size.x
                    || vertical * 9.0 >= state.view.scroll as f32 { 0 } else {
                    (state.view.scroll as f32 - vertical * 9.0)
                        .min((state.view.height as f32) - state.buffer.size.y as f32) as u32
                };
                state.update();
            });

            if state.dirty {
                state.dirty = false;
                View::draw(state);
                Some(std::mem::replace(&mut state.buffer, Buffer::new(Vec2::zero())))
            } else {
                None
            }
        };

        if let Some(buffer) = buffer_opt {
            window
                .update_with_buffer(&buffer.data, buffer.size.x as usize, buffer.size.y as usize)
                .unwrap();
            GLOBAL_STATE.write().unwrap().buffer = buffer;
        } else {
            window.update();
        }
    }
}
