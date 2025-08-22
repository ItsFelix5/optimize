use crate::buffer::{Buffer, BufferView};
use crate::{Vec2, State};
use image::{DynamicImage, GenericImageView, Limits};
use std::cell::RefCell;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::fs;
use allocative::Allocative;
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Allocative)]
pub struct Image {
    pub path: PathBuf,
    pub name: String,
    buffer: Arc<RwLock<Option<Buffer>>>,
    pub size: Vec2,
}

impl Image {
    pub fn get(&mut self, state: &State) -> Arc<RwLock<Option<Buffer>>> {
        let _ = self.buffer.write().and_then(|mut buf| {
            if buf.is_none() {
                *buf = Some(Buffer::empty());

                let pico = state.config.pico;
                let path = self.path.clone();
                let arc = self.buffer.clone();
                if pico {
                    Image::load(pico, path, arc);
                } else {
                    thread::spawn(move || {
                        Image::load(pico, path, arc);
                    });
                }
            }
            Ok(())
        });

        self.buffer.clone()
    }

    fn load(limits: bool, path: PathBuf, arc: Arc<RwLock<Option<Buffer>>>) {
        let mut image = match image::ImageReader::open(path).and_then(|img| img.with_guessed_format()) {
            Ok(reader) => reader,
            Err(err) => {
                println!("Failed to open image: {}", err);
                return;
            }
        };
        if limits {
            let mut limits = Limits::default();
            limits.max_alloc = Some(50 * 1024 * 1024);
            image.limits(limits);
        }
        let image = match image.decode() {
            Ok(img) => img,
            Err(err) => {
                println!("Failed to decode image: {}", err);
                return;
            }
        };
        let mut buf = match arc.write() {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let (w, h) = (image.width() as f32, image.height() as f32);
        let scale = (500.0 / w).min(500.0 / h);

        *buf = Some(image.scale(Vec2::from((w * scale).round() as u32, (h * scale).round() as u32)));
    }

    pub fn loaded(&self) -> bool {
        self.buffer.read().unwrap().is_some()
    }
}

impl BufferView for DynamicImage {
    fn size(&self) -> Vec2 {
        Vec2::from(DynamicImage::width(self), DynamicImage::height(self))
    }

    fn get(&self, pos: Vec2) -> u32 {
        let pixel = self.get_pixel(pos.x, pos.y).0;
        (0xFF << 24)
            | ((pixel[0] as u32) << 16)
            | ((pixel[1] as u32) << 8)
            | pixel[2] as u32
    }
}

#[derive(Allocative)]
pub struct Library {
    pub images: Vec<Image>,
}

impl Library {
    pub fn new() -> Library {
        Library { images: Vec::new() }
    }

    pub fn load(&mut self, mut dir: PathBuf) {
        if Path::new(&dir).is_relative() {
            if let Ok(home) = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }) {
                dir = PathBuf::from(home).join(dir);
            }
        }

        for result in match fs::read_dir(&dir) {
            Ok(content) => content,
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => {
                        let _ = fs::create_dir(dir);
                    }
                    ErrorKind::NotADirectory => {
                        let _ = self.load_file(dir);
                    }
                    _ => {}
                };
                return;
            }
        } {
            if let Ok(entry) = result {
                if entry.metadata().is_ok_and(|meta| meta.is_dir()) {
                    self.load(entry.path());
                } else {
                    self.load_file(entry.path());
                }
            }
        }
    }

    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Option<()> {
        let dimensions = image::ImageReader::open(&path).ok()?
            .with_guessed_format().ok()?
            .into_dimensions().ok()?;

        self.images.push(Image {
            path: path.as_ref().to_path_buf(),
            name: path.as_ref().file_name()?.to_string_lossy().into(),
            buffer: Arc::new(RwLock::new(None)),
            size: dimensions.into(),
        });
        Some(())
    }
}
