use crate::Vec2;
use allocative::Allocative;

pub trait BufferView {
    fn size(&self) -> Vec2;
    fn get(&self, pos: Vec2) -> u32;

    fn scale(&self, size: Vec2) -> Buffer {
        if size == self.size() {
            return self.clone();
        }
        let mut buf = Buffer::new(size);
        let mut tmp_line: Vec<(u8, u8, u8)> = vec![(0, 0, 0); self.size().x as usize];

        let weights_x = compute_weights(self.size().x, size.x);
        let weights_y = compute_weights(self.size().y, size.y);
        for out_y in 0..size.y {
            let (y_start, y_wts) = &weights_y[out_y as usize];

            for x in 0..self.size().x {
                let mut acc = (0, 0, 0);
                for (i, &wy) in y_wts.iter().enumerate() {
                    let color = self.get(Vec2::from(x, y_start + i as u32));
                    acc.0 += (((color >> 16) & 0xFF) as f32 * wy) as u8;
                    acc.1 += (((color >> 8) & 0xFF) as f32 * wy) as u8;
                    acc.2 += (((color) & 0xFF) as f32 * wy) as u8;
                }
                tmp_line[x as usize] = acc;
            }

            for out_x in 0..size.x {
                let (x_start, x_wts) = &weights_x[out_x as usize];
                let mut acc = (0, 0, 0);
                for (i, &wx) in x_wts.iter().enumerate() {
                    let color = tmp_line[*x_start as usize + i];
                    acc.0 += (color.0 as f32 * wx) as u8;
                    acc.1 += (color.1 as f32 * wx) as u8;
                    acc.2 += (color.2 as f32 * wx) as u8;
                }
                buf.set(Vec2::from(out_x, out_y),
                        (0xFF << 24) | ((acc.0 as u32) << 16) | ((acc.1 as u32) << 8) | acc.2 as u32,
                );
            }
        }
        buf
    }

    fn clone(&self) -> Buffer {
        let mut buf = Buffer::new(self.size());
        for x in 0..self.size().x {
            for y in 0..self.size().y {
                buf.set(Vec2::from(x, y), self.get(Vec2::from(x, y)));
            }
        }
        buf
    }

    fn rotate(&self) -> Buffer {
        let mut buf = Buffer::new(self.size());
        for x in 0..self.size().x {
            for y in 0..self.size().y {
                buf.set(Vec2::from(self.size().y - y - 1, x), self.get(Vec2::from(x, y)));
            }
        }
        buf
    }

    fn flip_g(&self) -> Buffer {
        let mut buf = Buffer::new(self.size());
        for x in 0..self.size().x {
            for y in 0..self.size().y {
                buf.set(Vec2::from(self.size().x - x - 1, y), self.get(Vec2::from(x, y)));
            }
        }
        buf
    }

    fn flip_v(&self) -> Buffer {
        let mut buf = Buffer::new(self.size());
        for x in 0..self.size().x {
            for y in 0..self.size().y {
                buf.set(Vec2::from(x, self.size().y - y - 1), self.get(Vec2::from(x, y)));
            }
        }
        buf
    }
}

#[derive(Allocative)]
pub struct Buffer {
    pub size: Vec2,
    pub data: Vec<u32>,
}

impl BufferView for Buffer {
    fn size(&self) -> Vec2 {
        self.size
    }

    fn get(&self, pos: Vec2) -> u32 {
        self.data[(pos.x as usize) + pos.y as usize * self.size.x as usize]
    }

    fn clone(&self) -> Buffer {
        Buffer {
            size: self.size,
            data: self.data.clone(),
        }
    }
}

impl Buffer {
    pub fn new(size: Vec2) -> Buffer {
        Buffer {
            size,
            data: vec![0xFFFFFFFF; size.x as usize * size.y as usize],
        }
    }

    pub fn empty() -> Buffer {
        Buffer {
            size: Vec2::zero(),
            data: Vec::new(),
        }
    }

    pub fn set(&mut self, pos: Vec2, color: u32) {
        if pos.x < self.size.x && pos.y < self.size.y {
            self.data[pos.x as usize + pos.y as usize * self.size.x as usize] = color;
        }
    }

    pub fn set_transparent(&mut self, pos: Vec2, color: u32, alpha: f32) {
        let original = self.get(pos);

        let r = (((color >> 16) & 0xFF) as f32 * alpha + ((original >> 16) & 0xFF) as f32 * (1.0 - alpha)) as u32;
        let g = (((color >> 8 ) & 0xFF) as f32 * alpha + ((original >> 8 ) & 0xFF) as f32 * (1.0 - alpha)) as u32;
        let b = (((color      ) & 0xFF) as f32 * alpha + ((original      ) & 0xFF) as f32 * (1.0 - alpha)) as u32;

        self.set(pos, (0xFF << 24) | (r << 16) | (g << 8) | b);
    }

    pub fn line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, thickness: u32, color: u32) {
        let (x0, y0, x1, y1) = (x1 as f32, y1 as f32, x2 as f32, y2 as f32);
        let dx = x1 - x0;
        let dy = y1 - y0;
        let length = (dx * dx + dy * dy).sqrt();
        if length == 0.0 {
            return;
        }

        let half = thickness as f32 / 2.0;

        let steps = length.ceil() as i32;
        for i in thickness as i32 / 2..=steps - thickness as i32 / 2 {
            let t = i as f32 / steps as f32;
            let px = x0 + t * dx;
            let py = y0 + t * dy;

            let radius = if i == 0 || i == steps {
                half
            } else {
                half - 0.5
            };
            let r2 = radius * radius;
            let mut minx = (px - radius).floor() as i32;
            let mut maxx = (px + radius).ceil() as i32;
            let mut miny = (py - radius).floor() as i32;
            let mut maxy = (py + radius).ceil() as i32;

            minx = minx.clamp(0, self.size.x as i32 - 1);
            maxx = maxx.clamp(0, self.size.x as i32 - 1);
            miny = miny.clamp(0, self.size.y as i32 - 1);
            maxy = maxy.clamp(0, self.size.y as i32 - 1);

            for sx in minx..=maxx {
                for sy in miny..=maxy {
                    let dx = sx as f32 - px;
                    let dy = sy as f32 - py;
                    let dist2 = dx * dx + dy * dy;
                    if dist2 <= r2 {
                        let t = (dist2 / r2).sqrt().clamp(0.0, 1.0);
                        let alpha = 1.0 - (t * t * (3.0 - 2.0 * t));
                        self.set_transparent(Vec2::from(sx as u32, sy as u32), color, alpha);
                    }
                }
            }
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.data.fill(color);
    }

    pub fn copy_from(&mut self, other: &impl BufferView, x: u32, y: i32) {
        for j in 0..other.size().y as i32 {
            if y + j > 0 {
                for i in 0..other.size().x {
                    self.set(
                        Vec2::from(x + i, (y + j) as u32),
                        other.get(Vec2::from(i, j as u32)),
                    );
                }
            }
        }
    }
}

fn compute_weights(src: u32, target: u32) -> Vec<(u32, Vec<f32>)> {
    let ratio = src as f32 / target as f32;
    let support = if ratio < 1.0 { 1.0 } else { ratio };
    let mut weights: Vec<(u32, Vec<f32>)> = Vec::with_capacity(target as usize);
    for out in 0..target {
        let input = (out as f32 + 0.5) * ratio;
        let start = ((input - support).floor() as i32).clamp(0, src as i32 - 1) as u32;
        let end = ((input + support).ceil() as i32).clamp(start as i32 + 1, src as i32) as u32;

        let mut wts = Vec::with_capacity((end - start) as usize);
        let mut sum = 0.0;
        for i in start..end {
            let x = ((i as f32 - input - 0.5) / support).abs();
            let w = if x < 1.0 { 1.0 - x } else { 0.0 };
            wts.push(w);
            sum += w;
        }
        if sum != 0.0 {
            for w in &mut wts {
                *w /= sum;
            }
        }
        weights.push((start, wts));
    }
    weights
}
