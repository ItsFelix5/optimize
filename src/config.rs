use std::path::Path;
use std::{fs, io};

pub struct Config {
    pub libraries: Vec<String>,
    pub background_color: u32,
    pub text_color: u32,
    pub primary_color: u32,
    pub secondary_color: u32,
    pub pico: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            libraries: vec!["Pictures".to_string()],
            background_color: 0xFF1A1A1A,
            text_color: 0xFFEEEEEE,
            primary_color: 0xFFEEEEEE,
            secondary_color: 0xFF4B4B4B,
            pico: false,
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    self.save(path);
                }
                return;
            }
        };

        self.libraries.clear();
        for line in content.lines() {
            if let Some((key, value)) = line.split_once(";") {
                match key {
                    "library" => self.libraries.push(value.to_string()),
                    "background_color" => {
                        let _ = u32::from_str_radix(&value[1..], 16)
                            .and_then(|c| Ok(self.background_color = c));
                    }
                    "text_color" => {
                        let _ = u32::from_str_radix(&value[1..], 16)
                            .and_then(|c| Ok(self.text_color = c));
                    }
                    "primary_color" => {
                        let _ = u32::from_str_radix(&value[1..], 16)
                            .and_then(|c| Ok(self.primary_color = c));
                    }
                    "secondary_color" => {
                        let _ = u32::from_str_radix(&value[1..], 16)
                            .and_then(|c| Ok(self.secondary_color = c));
                    }
                    "pico" => {
                        self.pico = true;
                    }
                    _ => {}
                };
            }
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let mut content = String::new();
        for lib in self.libraries.iter() {
            content.push_str(&*format!("library;#{lib}\n"));
        }
        content.push_str(&*format!("background_color;#{:X}\n", self.background_color));
        content.push_str(&*format!("text_color;#{:X}\n", self.text_color));
        content.push_str(&*format!("primary_color;#{:X}\n", self.primary_color));
        content.push_str(&*format!("secondary_color;#{:X}\n", self.secondary_color));
        if self.pico {
            content.push_str("pico;\n");
        }
        fs::write(path, content).unwrap_or_else(|err| eprintln!("Failed to save config: {}", err));
    }
}
