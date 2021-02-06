use std::collections::HashMap;
use std::time::Duration;

use tetra::graphics::{Mesh, Shader, Texture};
use tetra::Context;

use crate::sprite::AnimationMultiTextures;

/// Struct for a content data. One object per file.
#[derive(Clone)]
pub struct ContentPath {
    /// Key for access content after loaded
    pub key: String,
    /// Path to the file. Use it for load content.
    pub path: String,
}

impl ContentPath {
    #[must_use]
    pub const fn new(key: String, path: String) -> Self {
        Self { key, path }
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
    /// * `texture_loading_list` - List of texture files that need to load.
    ///
    #[must_use]
    pub fn new(texture_loading_list: Vec<ContentPath>) -> Self {
        Self {
            texture_id_counter: 1,
            texture_loading_list,
            textures: HashMap::new(),
            texture_ids: HashMap::new(),
            animations: HashMap::new(),
            animations_frame_length: HashMap::new(),
            shaders: HashMap::new(),
            meshes: HashMap::new(),
            tick: Duration::from_millis(0),
        }
    }

    #[must_use]
    pub fn hit_shader(&self) -> Option<&Shader> {
        self.shaders.get("hit-frame")
    }

    #[must_use]
    pub fn flash_red_shader(&self) -> Option<&Shader> {
        self.shaders.get("flash-red")
    }

    #[must_use]
    pub fn get_shader(&self, key: &str) -> Option<&Shader> {
        self.shaders.get(key)
    }

    #[must_use]
    pub fn get_id(&self, key: &str) -> u128 {
        self.texture_ids.get(key).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn get_by_id(&self, texture_id: &u128) -> Option<&Texture> {
        self.textures.get(texture_id)
    }

    pub fn load_animations(&mut self, animations: &[(&str, Vec<String>, u64)]) -> usize {
        if self.textures.is_empty() {
            panic!("ImageAssets try to load animation before load textures.");
        }
        for &(name, ref texture_names, num) in animations {
            let mut frame_ids = Vec::new();
            for texture_name in texture_names {
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

            self.animations.insert(name.to_owned(), frame_ids);
            self.animations_frame_length.insert(name.to_owned(), num);

            println!("Loaded animation: {}", name);
        }

        self.animations.len()
    }

    pub fn load_shaders(&mut self, ctx: &mut Context, shaders: &[(String, String)]) -> usize {
        let mut loaded_count = 0;
        for (shader, path) in shaders {
            match Shader::from_fragment_file(ctx, path.to_owned()) {
                Ok(v) => {
                    self.shaders.insert(shader.to_owned(), v);
                    loaded_count += 1;
                }
                Err(e) => panic!("Shader file missing. {}", e),
            };
        }

        loaded_count
    }

    #[must_use]
    pub fn get_all_animation_keys(&self) -> Vec<&String> {
        self.animations.keys().collect()
    }

    #[must_use]
    pub fn get_animation_frames(&self, animation_name: &str) -> Option<&Vec<u128>> {
        self.animations.get(animation_name)
    }

    #[must_use]
    pub fn get_animation_frame_length(&self, animation_name: &str) -> u64 {
        self.animations_frame_length
            .get(animation_name)
            .copied()
            .unwrap_or(15)
    }

    #[must_use]
    pub fn get_animation_object(&self, animation_name: &str) -> Option<AnimationMultiTextures> {
        if let Some(frames) = self.animations.get(animation_name) {
            let mut animation_object = AnimationMultiTextures::new_with_frames(&frames.clone());
            animation_object.name = animation_name.to_owned();
            animation_object.frame_length = Duration::from_millis(
                1000 / self.get_animation_frame_length(&animation_object.name),
            );
            return Some(animation_object);
        }

        None
    }

    pub fn add_content_list(&mut self, list: &[ContentPath]) {
        self.texture_loading_list.append(&mut list.to_vec());
    }

    pub fn add_content(&mut self, name: &str, path: &str) {
        self.texture_loading_list
            .push(ContentPath::new(name.to_owned(), path.to_owned()));
    }

    /// Is still loading?
    ///
    ///  # Return:
    ///
    /// True: still loading, False: loaded all contents
    ///
    #[must_use]
    pub fn is_loading(&self) -> bool {
        !self.texture_loading_list.is_empty()
    }

    /// Call function for loading one object from the list.
    /// Ideally, Call this function in `update()` and check until `is_loading()` return false.
    ///
    pub fn loading(&mut self, ctx: &mut Context) {
        if self.is_loading() {
            if self.shaders.is_empty() {
                // Load shader files here
            }

            let accumelated_time = tetra::time::get_accumulator(ctx).as_millis();
            if accumelated_time < 100 {
                self.tick += crate::ONE_FRAME;
            }

            if self.tick.as_millis() > 200 {
                for _count in 0..20 {
                    if let Some(content) = self.texture_loading_list.pop() {
                        match Texture::new(ctx, &content.path) {
                            Ok(v) => {
                                self.add(&content.key, v);

                                println!("Loaded \"{}\" : \"{}\"", content.key, content.path);
                            }
                            Err(e) => println!("Load texture error : {}", e),
                        };
                    };
                }

                self.tick = Duration::from_millis(0);
            }
        }
    }

    /// Add texture into texture bank with key
    fn add(&mut self, key: &str, texture: Texture) {
        self.texture_ids
            .insert(key.to_string(), self.texture_id_counter);
        self.textures.insert(self.texture_id_counter, texture);

        self.texture_id_counter += 1;
    }

    /// Get reference to texture in the texture bank by key
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Texture> {
        if let Some(id) = self.texture_ids.get(key) {
            return self.textures.get(id);
        }

        None
    }

    /// Clone texture in the texture bank by key
    fn make(&self, key: &str) -> Option<Texture> {
        self.get(key).cloned()
    }

    /// Clear all not loaded/loaded contents
    ///
    fn clear(&mut self) {
        self.texture_loading_list.clear();
        self.textures.clear();
        self.animations.clear();
    }

    pub fn add_mesh(&mut self, key: &str, mesh: Mesh) {
        self.meshes.insert(key.to_owned(), mesh);
    }

    #[must_use]
    pub fn get_mesh(&self, key: &str) -> Option<&Mesh> {
        self.meshes.get(key)
    }

    pub fn get_mut_mesh(&mut self, key: &str) -> Option<&mut Mesh> {
        self.meshes.get_mut(key)
    }
}
