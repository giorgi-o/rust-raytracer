use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    cameras::full_camera::FullCamera,
    core::{colour::Colour, transform::Transform, vector::Vector, vertex::Vertex},
    environments::{environment::Environment, photon_scene::PhotonScene, scene::Scene},
    lights::{
        directional_light::DirectionalLight, directional_point_light::DPLight, light::Light,
        point_light::PointLight,
    },
    materials::{
        compound_material::CompoundMaterial, falsecolour_material::FalseColour,
        global_material::GlobalMaterial, material::Material, phong_material::Monochrome,
        texture::Texture,
    },
    objects::{
        csg_object::{Csg, CsgMode},
        cuboid_object::Cuboid,
        object::Object,
        plane_object::Plane,
        polymesh_object::PolyMesh,
        quadratic_object::Quadratic,
        sphere_object::Sphere,
    },
};

type LineNumber = u32;

#[derive(Debug)]
pub struct ParseError {
    message: String,
    line: LineNumber,
}

type Result<T> = std::result::Result<T, ParseError>;

// err!(line_number, "...", ...)
macro_rules! err {
    ($line:expr, $($arg:tt)*) => {
        ParseError {
            message: format!($($arg)*),
            line: $line,
        }
    };
}

// bail!(line_number, "...", ...)
macro_rules! bail {
    ($line:expr, $($arg:tt)*) => {
        return Err(err!($line, $($arg)*))
    };
}

pub struct SceneFile {
    contents: String,
}

impl SceneFile {
    pub fn from_path(path: &PathBuf) -> Result<(Box<dyn Environment>, Box<FullCamera>)> {
        let contents = std::fs::read_to_string(path).expect("Failed to read scene file");
        Self::from_contents(contents)
    }

    pub fn from_contents(contents: String) -> Result<(Box<dyn Environment>, Box<FullCamera>)> {
        let paragraphs = Paragraph::parse_whole_file(contents)?;

        let (scenes, paragraphs): (Vec<_>, Vec<_>) =
            paragraphs.into_iter().partition(|p| p.is_scene());

        let mut scenes = scenes.into_iter();
        let mut scene = match scenes.next() {
            Some(scene) => {
                let ParagraphItem::Env(scene) = scene.into_item()? else {
                    panic!("is_scene() is true but into_item() is not Env")
                };
                scene
            }
            None => Box::new(Scene::new()),
        };

        if let Some(paragraph) = scenes.next() {
            bail!(paragraph.start_line, "Multiple scenes in file");
        }

        let mut camera = None;
        for paragraph in paragraphs {
            let start_line = paragraph.start_line;
            let item = paragraph.into_item()?;
            match item {
                ParagraphItem::Light(light) => scene.add_light(light),
                ParagraphItem::Object(object) => scene.add_object(object),
                ParagraphItem::Camera(c) => {
                    if camera.is_some() {
                        bail!(start_line, "Multiple cameras in file")
                    }
                    camera = Some(c)
                }
                ParagraphItem::Material(_) => {
                    bail!(start_line, "Cannot add material to scene on its own")
                }
                ParagraphItem::Env(_) => {
                    panic!("is_scene() is false but into_item() is Env")
                }
            }
        }

        let camera = camera.unwrap_or_else(|| {
            let width = 1024;
            let height = 1024;

            // "default" camera position
            let position = Vertex::new(0.0, 3.0, 0.0);
            let lookat = Vector::new(0.0, 0.5, 1.0).normalised();
            let up = Vector::new(0.0, lookat.z, -lookat.y);
            let fov = 40f32.to_radians();

            let camera = FullCamera::new(width, height, fov, position, lookat, up);
            Box::new(camera)
        });
        Ok((scene, camera))
    }
}

struct Paragraph {
    kind: String,
    class: String,
    attributes: HashMap<String, Attribute>,
    start_line: LineNumber,
}

impl Paragraph {
    fn parse_whole_file(contents: String) -> Result<Vec<Self>> {
        let mut paragraphs = Vec::new();

        let mut lines = contents.lines().enumerate();
        let mut lines_in_paragraph = vec![];
        let mut paragraph_start_line: LineNumber = 0;

        let mut process_paragraph =
            |lines_in_paragraph: &mut Vec<&str>, line_number: LineNumber| -> Result<()> {
                if lines_in_paragraph.is_empty() {
                    return Ok(());
                }

                let lines_in_paragraph = std::mem::take(lines_in_paragraph);
                let paragraph = Paragraph::parse(lines_in_paragraph, line_number)?;
                paragraphs.push(paragraph);
                Ok(())
            };

        loop {
            let next_line = lines.next();
            let Some((line_number, next_line)) = next_line else {
                process_paragraph(&mut lines_in_paragraph, paragraph_start_line)?;
                break;
            };
            let line_number = line_number as LineNumber + 1;

            if next_line.trim().is_empty() || next_line.starts_with('#') {
                process_paragraph(&mut lines_in_paragraph, paragraph_start_line)?;
                continue;
            }

            let indentation = next_line.chars().take_while(|c| c.is_whitespace()).count();
            if lines_in_paragraph.is_empty() && indentation == 0 {
                lines_in_paragraph.push(next_line);
                paragraph_start_line = line_number;
            } else if !lines_in_paragraph.is_empty() && indentation > 0 {
                lines_in_paragraph.push(next_line);
            } else {
                bail!(line_number, "Started next paragraph without empty newline");
            }
        }

        Ok(paragraphs)
    }

    fn parse(lines: Vec<&str>, first_line_number: LineNumber) -> Result<Self> {
        let first_line = lines[0];

        let get_indentation = |s: &str| s.chars().take_while(|c| c.is_whitespace()).count();
        let indentation = get_indentation(first_line);

        let mut words = first_line.split_whitespace();
        let kind = words
            .next()
            .ok_or_else(|| err!(first_line_number, "Empty paragraph"))?
            .to_string();
        let class = words
            .next()
            .ok_or_else(|| err!(first_line_number, "Missing paragraph class"))?
            .to_string();
        // if words.next().is_some() {
        if let Some(word) = words.next() {
            bail!(
                first_line_number,
                "Too many words in paragraph header: {}",
                word
            );
        }

        let mut attributes = HashMap::new();
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() || get_indentation(line) <= indentation {
                // end of paragraph
                break;
            }

            let line_number = first_line_number + i as LineNumber;

            let mut words = line.split_whitespace();
            let key = words.next().unwrap();
            if key.starts_with('#') {
                continue; // comment
            }

            let words: Vec<&str> = words.collect();

            let value = if words.len() == 3 {
                let parse_error = || err!(line_number, "Invalid float");
                let x = words[0].parse::<f32>().map_err(|_| parse_error())?;
                let y = words[1].parse::<f32>().map_err(|_| parse_error())?;
                let z = words[2].parse::<f32>().map_err(|_| parse_error())?;
                AttributeValue::Vector(Vector::new(x, y, z))
            } else if words.len() == 1 {
                // if the next line is more indented
                if i + 1 < lines.len() && get_indentation(lines[i + 1]) > get_indentation(line) {
                    // this is a sub-paragraph
                    let p = Paragraph::parse(lines[i..].to_vec(), line_number);
                    AttributeValue::SubParagraph(Box::new(p?))
                } else {
                    // either a float or a word
                    match words[0].parse::<f32>() {
                        Ok(f) => AttributeValue::Float(f),
                        Err(_) => AttributeValue::Word(words[0].to_string()),
                    }
                }
            } else {
                bail!(
                    line_number,
                    "Invalid word count in attribute value: {}",
                    line
                );
            };

            let key = key.to_string();
            let attribute = Attribute {
                key: key.clone(),
                value,
                line_number,
            };
            attributes.insert(key, attribute);
        }

        Ok(Self {
            kind,
            class,
            attributes,
            start_line: first_line_number,
        })
    }

    fn into_item(self) -> Result<ParagraphItem> {
        match self.kind.as_str() {
            "light" => Ok(ParagraphItem::Light(self.into_light()?)),
            "object" => Ok(ParagraphItem::Object(self.into_object()?)),
            "material" => Ok(ParagraphItem::Material(self.into_material()?)),
            "scene" => Ok(ParagraphItem::Env(self.into_scene()?)),
            "camera" => Ok(ParagraphItem::Camera(self.into_camera()?)),
            _ => bail!(self.start_line, "Invalid paragraph kind: {}", self.kind),
        }
    }

    fn is_scene(&self) -> bool {
        self.kind == "scene"
    }

    fn into_scene(self) -> Result<Box<dyn Environment>> {
        let scene: Box<dyn Environment> = match self.class.as_str() {
            "Scene" => Box::new(Scene::new()),
            "PhotonScene" => Box::new(PhotonScene::new()),
            _ => bail!(self.start_line, "Invalid scene class: {}", self.class),
        };
        Ok(scene)
    }

    fn into_light(mut self) -> Result<Box<dyn Light>> {
        let light: Box<dyn Light> = match self.class.as_str() {
            "Directional" => DirectionalLight::new(
                self.get_attr("direction")?.as_vector()?,
                self.get_attr_or("colour", AttributeValue::Float(1.0))
                    .as_colour()?,
            ),
            "Point" => PointLight::new(
                self.get_attr("position")?.as_vertex()?,
                self.get_attr_or("colour", AttributeValue::Float(1.0))
                    .as_colour()?,
            ),
            "DirPoint" => DPLight::new(
                self.get_attr("position")?.as_vertex()?,
                self.get_attr("direction")?.as_vector()?,
                self.get_attr_or("colour", AttributeValue::Float(1.0))
                    .as_colour()?,
            ),
            _ => bail!(self.start_line, "Invalid light class: {}", self.class),
        };
        Ok(light)
    }

    fn into_object(mut self) -> Result<Box<dyn Object>> {
        let object: Box<dyn Object> = match self.class.as_str() {
            "Plane" => Plane::new(
                &self.get_attr("point")?.as_vertex()?,
                self.get_attr("up")?.as_vector()?,
                self.get_attr("normal")?.as_vector()?,
                self.get_attr("material")?.into_material()?,
            ),
            "Sphere" => Sphere::new(
                self.get_attr("centre")?.as_vertex()?,
                self.get_attr("radius")?.as_float()?,
                self.get_attr("material")?.into_material()?,
            ),
            "Cuboid" => Cuboid::new(
                self.get_attr("corner")?.as_vertex()?,
                self.get_attr("size")?.as_vector()?,
                self.get_attr("material")?.into_material()?,
            ),
            "Quadratic" => {
                let mut quadratic = Quadratic::new(
                    (
                        self.get_attr_or("a", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("b", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("c", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("d", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("e", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("f", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("g", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("h", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("i", AttributeValue::Float(0.0))
                            .as_float()?,
                        self.get_attr_or("j", AttributeValue::Float(0.0))
                            .as_float()?,
                    ),
                    self.get_attr("material")?.into_material()?,
                );
                if let Ok(transform) = self.get_attr("translate") {
                    let transform = transform.as_vector()?;
                    let transform = Transform::from_translation(transform);
                    quadratic.apply_transform(&transform);
                }
                quadratic
            }
            "Model" => {
                let obj_path = self.get_attr("obj")?.as_word()?;
                let obj_path = PathBuf::from("assets").join("models").join(obj_path);
                let mut model = PolyMesh::from_obj_file(
                    obj_path,
                    self.get_attr("material")?.into_material()?,
                    self.get_attr_or("smooth", AttributeValue::Float(0.0))
                        .as_float()?
                        != 0.0,
                );

                // tmp: special fix for teapot model
                if self.get_attr("rotate_teapot").is_ok() {
                    // rotate the teapot by 90 degrees
                    let rotation = Transform::from_rotation_matrix([
                        [1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0],
                        [0.0, -1.0, 0.0],
                    ]);
                    model.apply_transform(&rotation);
                }

                let mut transform = Transform::identity();

                if let Some(translate) = self.attributes.remove("translate") {
                    let translate = translate.as_vector()?;
                    transform[3][0] = translate.x;
                    transform[3][1] = translate.y;
                    transform[3][2] = translate.z;
                }

                if let Some(scale) = self.attributes.remove("scale") {
                    let scale = scale.as_float()?;
                    transform[0][0] = scale;
                    transform[1][1] = scale;
                    transform[2][2] = scale;
                }

                model.apply_transform(&transform.transposed());

                Box::new(model)
            }
            "MarioPipe" => {
                let material = self.get_attr("material")?.into_material()?;

                let main_cylinder =
                    // Quadratic::new((4., 0., 0., 0., 0., 0., 0., 4., 0., -1.), material.clone());
                    Quadratic::cylinder(1., material.clone());
                let main_cylinder_box = Cuboid::new(
                    Vertex::new(-0.5, 0., -0.5),
                    Vector::new(1.0, 3.0, 1.0),
                    material.clone(),
                );
                let main_cylinder =
                    Csg::new(CsgMode::Intersection, main_cylinder, main_cylinder_box);

                let top_cylinder = Quadratic::cylinder(1.3, material.clone());
                let top_cylinder_box = Cuboid::new(
                    Vertex::new(-0.7, 2.7, -0.7),
                    Vector::new(1.3, 0.3, 1.3),
                    material.clone(),
                );
                let top_cylinder = Csg::new(CsgMode::Intersection, top_cylinder, top_cylinder_box);
                let mut pipe = Csg::new(CsgMode::Union, main_cylinder, top_cylinder);

                // thin black sphere slice on the top to give the illusion of a hole
                let black_hole = Sphere::new(
                    Vertex::new(0., 3., 0.),
                    0.5,
                    Monochrome::new(Colour::black(), 0.1, 100000000.0),
                );
                let black_hole_box = Cuboid::new(
                    Vertex::new(-0.5, 3.0, -0.5),
                    Vector::new(1.0, 0.0001, 1.0),
                    Monochrome::new(Colour::black(), 0.1, 100000000.0),
                );
                let black_hole = Csg::new(CsgMode::Intersection, black_hole, black_hole_box);
                pipe = Csg::new(CsgMode::Union, pipe, black_hole);

                let mut transform = Transform::identity();
                if let Some(scale) = self.attributes.remove("scale") {
                    let scale = scale.as_float()?;
                    transform[0][0] = scale;
                    transform[1][1] = scale;
                    transform[2][2] = scale;
                }
                if let Some(translate) = self.attributes.remove("translate") {
                    let translate = translate.as_vector()?;
                    transform[3][0] = translate.x;
                    transform[3][1] = translate.y;
                    transform[3][2] = translate.z;
                }
                pipe.apply_transform(&transform.transposed());
                pipe
            }
            "MarioCoin" => {
                let material = self.get_attr("material")?.into_material()?;

                let coin = Sphere::new(Vertex::new(0., 0., 0.), 0.5, material.clone());
                let coin_box = Cuboid::new(
                    Vertex::new(-0.5, -0.5, -0.1),
                    Vector::new(1., 1., 0.2),
                    material.clone(),
                );
                let mut coin = Csg::new(CsgMode::Intersection, coin, coin_box);

                // shallow part
                let shallow_part = Sphere::new(Vertex::new(0., 0., 0.), 0.4, material.clone());
                let shallow_part_box = Cuboid::new(
                    Vertex::new(-0.4, -0.4, -0.11),
                    Vector::new(0.8, 0.8, 0.03),
                    material.clone(),
                );
                let shallow_part = Csg::new(CsgMode::Intersection, shallow_part, shallow_part_box);
                coin = Csg::new(CsgMode::Difference, coin, shallow_part);

                // line decoration
                let line = Cuboid::new(
                    Vertex::new(-0.08, -0.28, -0.1),
                    Vector::new(0.16, 0.56, 0.05),
                    material.clone(),
                );
                coin = Csg::new(CsgMode::Difference, coin, line);

                if let Some(scale) = self.attributes.remove("scale") {
                    let scale = scale.as_float()?;
                    let mut transform = Transform::identity();
                    transform[0][0] = scale;
                    transform[1][1] = scale;
                    transform[2][2] = scale;
                    coin.apply_transform(&transform.transposed());
                }
                if let Some(translate) = self.attributes.remove("translate") {
                    let mut transform = Transform::identity();
                    let translate = translate.as_vector()?;
                    transform[3][0] = translate.x;
                    transform[3][1] = translate.y;
                    transform[3][2] = translate.z;
                    coin.apply_transform(&transform.transposed());
                }

                coin
            }
            _ => bail!(self.start_line, "Invalid object class: {}", self.class),
        };
        Ok(object)
    }

    fn into_material(mut self) -> Result<Arc<dyn Material>> {
        let material: Arc<dyn Material> = match self.class.as_str() {
            "Simple" => CompoundMaterial::new_simple(
                self.get_attr("colour")?.as_colour()?,
                self.get_attr("reflectiveness")?.as_float()?,
                self.get_attr_or("shininess", AttributeValue::Float(100.0))
                    .as_float()?,
            ),
            "Transparent" => CompoundMaterial::new_translucent(
                self.get_attr("colour")?.as_colour()?,
                self.get_attr("transparency")?.as_float()?,
                self.get_attr("ior")?.as_float()?,
                self.get_attr_or("shininess", AttributeValue::Float(100.0))
                    .as_float()?,
            ),
            "Global" => GlobalMaterial::new(
                self.get_attr("reflect")?.as_float()?,
                self.get_attr("refract")?.as_float()?,
                self.get_attr("ior")?.as_float()?,
            ),
            "Monochrome" => Monochrome::new(
                self.get_attr("colour")?.as_colour()?,
                self.get_attr_or("ambient", AttributeValue::Float(0.1))
                    .as_float()?,
                self.get_attr("shininess")?.as_float()?,
            ),
            // "Texture" => Texture::import(name, scale, ambient_strength, shininess)
            "Texture" => Texture::import(
                self.get_attr("name")?.as_word()?,
                self.get_attr("scale")?.as_float()?,
                self.get_attr("ambient")?.as_float()?,
                self.get_attr("shininess")?.as_float()?,
            ),
            "TransparentTexture" => CompoundMaterial::new_textured(
                self.get_attr("name")?.as_word()?,
                self.get_attr("scale")?.as_float()?,
                self.get_attr("transparency")?.as_float()?,
            ),
            "FalseColour" => Arc::new(FalseColour::new()),
            _ => bail!(self.start_line, "Invalid material class: {}", self.class),
        };
        Ok(material)
    }

    fn into_camera(mut self) -> Result<Box<FullCamera>> {
        if self.class != "Camera" {
            bail!(self.start_line, "Invalid camera class: {}", self.class);
        }

        let res = self.get_attr_or("res", AttributeValue::Float(1024.0)).value;
        let width = self.get_attr_or("width", res).as_float()?;
        let height = self
            .get_attr_or("height", AttributeValue::Float(width))
            .as_float()?;
        let fov = self
            .get_attr_or("fov", AttributeValue::Float(40.0))
            .as_float()?
            .to_radians();

        let position = self
            .get_attr_or(
                "position",
                AttributeValue::Vector(Vector::new(0.0, 0.0, 0.0)),
            )
            .as_vertex()?;
        let lookat = self
            .get_attr_or("lookat", AttributeValue::Vector(Vector::new(0.0, 0.0, 1.0)))
            .as_vector()?;
        let up = self
            .get_attr_or(
                "up",
                AttributeValue::Vector(Vector::new(0.0, lookat.z, -lookat.y)),
            )
            .as_vector()?;

        let camera = FullCamera::new(width as u32, height as u32, fov, position, lookat, up);
        Ok(Box::new(camera))
    }

    fn get_attr(&mut self, key: &str) -> Result<Attribute> {
        self.attributes
            .remove(key)
            .ok_or_else(|| err!(self.start_line, "Missing required attribute: {}", key))
    }

    fn get_attr_or(&mut self, key: &str, default: AttributeValue) -> Attribute {
        self.attributes.remove(key).unwrap_or(Attribute {
            key: key.to_string(),
            value: default,
            line_number: 0,
        })
    }
}

struct Attribute {
    key: String,
    value: AttributeValue,
    line_number: LineNumber,
}

enum AttributeValue {
    Word(String),
    Float(f32),
    Vector(Vector),
    SubParagraph(Box<Paragraph>),
}

impl Attribute {
    fn as_word(&self) -> Result<String> {
        Ok(match &self.value {
            AttributeValue::Word(w) => w.clone(),
            _ => bail!(self.line_number, "Invalid attribute value for word"),
        })
    }

    fn as_colour(&self) -> Result<Colour> {
        Ok(match &self.value {
            AttributeValue::Vector(v) => Colour::new(v.x, v.y, v.z),
            AttributeValue::Float(f) => Colour::new(*f, *f, *f),
            AttributeValue::Word(w) => match w.as_str() {
                "White" => Colour::white(),
                "Black" => Colour::black(),
                _ => bail!(self.line_number, "Unknown colour name: {}", w),
            },
            _ => bail!(self.line_number, "Invalid attribute value for colour"),
        })
    }

    fn as_vector(&self) -> Result<Vector> {
        Ok(match self.value {
            AttributeValue::Vector(v) => v,
            AttributeValue::Float(f) => Vector::new(f, f, f),
            _ => bail!(self.line_number, "Invalid attribute value for vector"),
        })
    }

    fn as_vertex(&self) -> Result<Vertex> {
        let vector = self.as_vector()?;
        Ok(Vertex::new(vector.x, vector.y, vector.z))
    }

    fn as_float(&self) -> Result<f32> {
        Ok(match self.value {
            AttributeValue::Float(f) => f,
            _ => bail!(self.line_number, "Invalid attribute value for float"),
        })
    }

    fn into_material(self) -> Result<Arc<dyn Material>> {
        let AttributeValue::SubParagraph(p) = self.value else {
            bail!(self.line_number, "Invalid attribute value for material");
        };
        p.into_material()
    }
}

enum ParagraphItem {
    Env(Box<dyn Environment>),
    Camera(Box<FullCamera>),
    Light(Box<dyn Light>),
    Object(Box<dyn Object>),
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
