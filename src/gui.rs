use crate::{Vec2, State};

pub struct View {
    widgets: Vec<Box<dyn Widget>>,
    pub height: u32,
    pub scroll: u32,
}

impl View {
    pub fn gallery() -> Self {
        let mut widgets: Vec<Box<dyn Widget>> = Vec::new();
        widgets.push(Box::new(Scrollbar::default()));
        widgets.push(Box::new(Gallery::default()));
        View {
            widgets,
            height: 0,
            scroll: 0,
        }
    }

    pub fn resize(state: &mut State) {
        let mut widgets = std::mem::take(&mut state.view.widgets);
        for widget in widgets.iter_mut() {
            widget.resize(state);
        }
        state.view.widgets = widgets;
    }

    pub fn draw(state: &mut State) {
        state.buffer.clear(state.config.background_color);
        let mut widgets = std::mem::take(&mut state.view.widgets);
        for widget in widgets.iter_mut() {
            widget.draw(state);
        }
        state.view.widgets = widgets;
    }
}

pub trait Widget {
    fn pos(&self, scroll: u32) -> Vec2;
    fn size(&self) -> Vec2;
    fn resize(&mut self, _state: &mut State) {}
    fn draw(&mut self, state: &mut State);
}

#[derive(Default)]
struct Gallery {
    viewport: Vec2,
    children: Vec<Image>,
}

impl Widget for Gallery {
    fn pos(&self, _scroll: u32) -> Vec2 {
        Vec2::from(20, 20)
    }

    fn size(&self) -> Vec2 {
        self.viewport.sub(40)
    }

    fn resize(&mut self, state: &mut State) {
        self.viewport = state.buffer.size;

        if self.children.is_empty() {
            for index in 0..state.library.images.len() {
                let mut img = Image {
                    pos: Vec2::zero(),
                    size: Vec2::zero(),
                    index,
                };
                img.resize(state);
                self.children.push(img);
            }
        }

        let mut left = 20;
        let mut top = 20i32;
        let mut row_height = 0;
        for image in &mut self.children {
            let size = image.size();
            if left + size.x > self.viewport.x - 20 {
                left = 20;
                top += (row_height + 20) as i32;
                row_height = 0;
            }
            if size.y > row_height {
                row_height = size.y;
            }

            image.pos = Vec2::from(left, top as u32);
            left += size.x + 20;
        }
        state.view.height = top as u32 + row_height + 20;
    }

    fn draw(&mut self, state: &mut State) {
        for widget in self.children.iter_mut() {
            let y = (widget.pos.y as i32) - state.view.scroll as i32;
            if y > -(widget.size().y as i32) && y < self.viewport.y as i32 {
                widget.draw(state);
            }
        }
    }
}

struct Image {
    index: usize,
    pos: Vec2,
    size: Vec2,
}

impl Widget for Image {
    fn pos(&self, scroll: u32) -> Vec2 {
        self.pos.sub2(0, scroll)
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn resize(&mut self, state: &mut State) {
        self.size = state.library.images[self.index].size;
    }

    fn draw(&mut self, state: &mut State) {
        let mut image = state.library.images.remove(self.index);
        let arc = image.get(state);
        if let Ok(buf) = arc.try_read() {
            if let Some(ref buf) = *buf {
                state
                    .buffer
                    .copy_from(buf, self.pos.x, self.pos.y as i32 - state.view.scroll as i32);
            }
        }

        state.library.images.insert(self.index, image);
    }
}

#[derive(Default)]
struct Scrollbar {
    viewport: Vec2,
}

impl Widget for Scrollbar {
    fn pos(&self, _scroll: u32) -> Vec2 {
        Vec2::from(self.viewport.x - 10, 0)
    }
    fn size(&self) -> Vec2 {
        Vec2::from(10, self.viewport.y)
    }
    fn resize(&mut self, state: &mut State) {
        self.viewport = state.buffer.size;
    }

    fn draw(&mut self, state: &mut State) {
        let viewport = self.viewport.y as f32 - 1.0;
        let height = state.view.height as f32;
        if height <= viewport {
            return;
        }

        let x = self.viewport.x - 5;
        let length = (viewport / height * viewport).max(10.0);

        let max_scroll = (height - viewport).max(1.0);
        let max_y = (viewport - length).max(0.0);

        let y = ((state.view.scroll as f32 / max_scroll) * max_y).clamp(0.0, max_y);
        state.buffer.line(
            x,
            y as u32,
            x,
            (y + length) as u32,
            10,
            state.config.secondary_color,
        );
    }
}
