use std::{path::PathBuf, sync::Arc, collections::HashMap};

use crate::{materials::material::Material, core::{vector::Vector, colour::Colour}};

struct SceneFile {
    contents: String,
}

impl SceneFile {
    pub fn from_path(path: &PathBuf) -> Self {
        let contents = std::fs::read_to_string(path).expect("Failed to read scene file");
        Self::from_contents(contents)
    }

    pub fn from_contents(contents: String) -> Self {
        Self { contents }
    }
}

struct Paragraph {
    kind: String,
    class: String,
    attributes: HashMap<String, AttributeValue>,
}

impl Paragraph {
    fn parse(str: String) -> Self {
        let mut lines = str.split('\n');
        let first_line = lines.next().unwrap();
        let indentation = first_line.chars().take_while(|c| c.is_whitespace()).count();

        let mut words = first_line.split_whitespace();
        let kind = words.next().unwrap();
        let class = words.next().unwrap();
        assert!(words.next().is_none());

        // let mut attributes = HashMap::new();
        for line in lines {
            let mut words = line.split_whitespace();
            let key = words.next().unwrap();
            let words: Vec<&str> = words.collect();

            
        }

        todo!()
    }
}

enum AttributeValue {
    Float(f32),
    Vector(Vector),
    Colour(Colour),
    Material(Arc<dyn Material>),
}

struct SceneFileParagraphs {
    file: SceneFile,
}

impl Iterator for SceneFileParagraphs {
    type Item = Paragraph;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

