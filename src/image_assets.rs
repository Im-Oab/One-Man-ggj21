use std::collections::HashMap;
use std::time::Duration;

use tetra::graphics::{Mesh, Shader, Texture};
use tetra::Context;

use crate::sprite::AnimationMultiTextures;

/// Struct for a content data. One object per file.
pub struct ContentPath {
    /// Key for access content after loaded
    pub key: String,
    /// Path to the file. Use it for load content.
    pub path: String,
}

impl ContentPath {
    pub fn new(key: String, path: String) -> ContentPath {
        ContentPath {
            key: key,
            path: path,
        }
    }
}

impl Clone for ContentPath {
    fn clone(&self) -> ContentPath {
        ContentPath {
            key: self.key.clone(),
            path: self.path.clone(),
        }
    }
}

/// Use for keepping list of contents that need to load.
/// After loaded. Content can access using "key"
///
pub struct ImageAssets {
    texture_id_counter: u128,
    /// Keep all ContentPath related to Texture. If this list empty then no texture to load.
    texture_loading_list: Vec<ContentPath>,
    /// Keep all Textures. It will populated from texture_loading_list.
    textures: HashMap<u128, Texture>,

    texture_ids: HashMap<String, u128>,

    animations: HashMap<String, Vec<u128>>,

    animations_frame_length: HashMap<String, u64>,

    shaders: HashMap<String, Shader>,

    tick: Duration,

    meshes: HashMap<String, Mesh>,
}

impl ImageAssets {
    /// Create object with list of contents that need to load.
    ///
    /// Currently, support only loading Texture
    /// # Arguments:
    ///
    /// * texture_loading_list - List of texture files that need to load.
    ///
    pub fn new(texture_loading_list: Vec<ContentPath>) -> ImageAssets {
        ImageAssets {
            texture_id_counter: 1,
            texture_loading_list: texture_loading_list,
            textures: HashMap::new(),
            texture_ids: HashMap::new(),
            animations: HashMap::new(),
            animations_frame_length: HashMap::new(),
            shaders: HashMap::new(),
            meshes: HashMap::new(),
            tick: Duration::from_millis(0),
        }
    }

    pub fn hit_shader(&self) -> Option<&Shader> {
        self.shaders.get("hit-frame")
    }

    pub fn flash_red_shader(&self) -> Option<&Shader> {
        self.shaders.get("flash-red")
    }

    pub fn get_shader(&self, key: &str) -> Option<&Shader> {
        self.shaders.get(key)
    }

    pub fn get_id(&self, key: &String) -> u128 {
        match self.texture_ids.get(key) {
            Some(id) => *id,
            None => 0,
        }
    }

    pub fn get_by_id(&self, texture_id: &u128) -> Option<&Texture> {
        self.textures.get(texture_id)
    }

    pub fn load_animations(&mut self, animations: &Vec<(&str, Vec<String>, u64)>) -> usize {
        if self.textures.len() == 0 {
            panic!("ImageAssets try to load animation before load textures.");
        }
        for anim in animations.iter() {
            let name = anim.0.clone();
            let mut frame_ids = Vec::new();
            for texture_name in anim.1.iter() {
                let texture_id = self.get_id(texture_name);
                if texture_id > 0 {
                    frame_ids.push(texture_id);
                } else {
                    println!(
                        "load_animations()::Texture missing ({}) for {}",
                        texture_name, name
                    );
                }
            }

            self.animations.insert(String::from(name), frame_ids);
            self.animations_frame_length
                .insert(String::from(anim.0), anim.2);

            println!("Loaded animation: {}", anim.0);
        }

        self.animations.len()
    }

    pub fn load_shaders(&mut self, ctx: &mut Context, shaders: &Vec<(String, String)>) -> usize {
        let mut loaded_count = 0;
        for shader_path in shaders.iter() {
            match Shader::from_fragment_file(ctx, shader_path.1.to_owned()) {
                Ok(v) => {
                    self.shaders.insert(shader_path.0.to_owned(), v);
                    loaded_count += 1;
                }
                Err(e) => panic!("Shader file missing. {}", e),
            };
        }

        loaded_count
    }

    pub fn get_all_animation_keys(&self) -> Vec<&String> {
        self.animations.keys().collect()
    }

    pub fn get_animation_frames(&self, animation_name: &String) -> Option<&Vec<u128>> {
        self.animations.get(animation_name)
    }

    pub fn get_animation_frame_length(&self, animation_name: &String) -> u64 {
        match self.animations_frame_length.get(animation_name) {
            Some(v) => *v,
            None => 15,
        }
    }

    pub fn get_animation_object(&self, animation_name: &str) -> Option<AnimationMultiTextures> {
        match self.animations.get(animation_name) {
            Some(frames) => {
                let mut animation_object = AnimationMultiTextures::new_with_frames(frames.clone());
                animation_object.name = String::from(animation_name);
                animation_object.frame_length = Duration::from_millis(
                    1000 / self.get_animation_frame_length(&animation_object.name),
                );
                return Some(animation_object);
            }
            None => (),
        }

        None
    }

    pub fn add_content_list(&mut self, list: &Vec<ContentPath>) {
        for ref_node in list.iter() {
            self.texture_loading_list.push(ref_node.clone());
        }
    }

    pub fn add_content(&mut self, name: &str, path: &str) {
        self.texture_loading_list
            .push(ContentPath::new(String::from(name), String::from(path)));
    }

    /// Is still loading?
    ///
    ///  # Return:
    ///
    /// True: still loading, False: loaded all contents
    ///
    pub fn is_loading(&self) -> bool {
        self.texture_loading_list.len() > 0
    }

    /// Call function for loading one object from the list.
    /// Ideally, Call this function in update() and check until is_loading() return false.
    ///
    pub fn loading(&mut self, ctx: &mut Context) {
        if self.is_loading() {
            if self.shaders.len() == 0 {
                // Load shader files here
            }

            let accumelated_time = tetra::time::get_accumulator(ctx).as_millis();
            if accumelated_time < 100 {
                self.tick += crate::ONE_FRAME;
            }

            if self.tick.as_millis() > 200 {
                for _count in 0..20 {
                    match self.texture_loading_list.pop() {
                        Some(content) => {
                            match Texture::new(ctx, &content.path) {
                                Ok(v) => {
                                    self.add(&content.key, v);

                                    println!("Loaded \"{}\" : \"{}\"", content.key, content.path);
                                }
                                Err(e) => println!("Load texture error : {}", e),
                            };
                        }
                        None => (),
                    };
                }

                self.tick = Duration::from_millis(0);
            }
        }
    }

    /// Add texture into texture bank with key
    fn add(&mut self, key: &String, texture: Texture) {
        self.texture_ids
            .insert(key.to_string(), self.texture_id_counter);
        self.textures.insert(self.texture_id_counter, texture);

        self.texture_id_counter += 1;
    }

    /// Get reference to texture in the texture bank by key
    pub fn get(&self, key: &str) -> Option<&Texture> {
        match self.texture_ids.get(key) {
            Some(id) => {
                return self.textures.get(id);
            }
            None => (),
        }

        None
    }

    /// Clone texture in the texture bank by key
    fn make(&self, key: &String) -> Option<Texture> {
        match self.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    /// Clear all not loaded/loaded contents
    ///
    fn clear(&mut self) {
        self.texture_loading_list.clear();
        self.textures.clear();
        self.animations.clear();
    }

    pub fn add_mesh(&mut self, key: &str, mesh: Mesh) {
        self.meshes.insert(String::from(key), mesh);
    }

    pub fn get_mesh(&self, key: &str) -> Option<&Mesh> {
        self.meshes.get(key)
    }

    pub fn get_mut_mesh(&mut self, key: &str) -> Option<&mut Mesh> {
        self.meshes.get_mut(key)
    }
}
