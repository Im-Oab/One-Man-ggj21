use std::collections::{HashMap, VecDeque};

use tetra::math::Vec2;

use crate::sprite::Sprite;

pub struct NodePoint {
    pub name: String,
    pub position: Vec2<f32>,
    pub waiting_time: u128,
    pub next_node: Option<String>,
    pub spawn_patterns: VecDeque<PatternNode>,
}

pub struct Level {
    is_start: bool,
    pub backgrounds: Vec<Sprite>,
    current_camera_target_node_name: Option<String>,
    all_nodes: HashMap<String, NodePoint>,
    patterns: HashMap<String, VecDeque<EnemySpawnNode>>,

    spawn_duration: u128,
    current_node_spawn_patterns: VecDeque<PatternNode>,
    active_patterns: Vec<VecDeque<EnemySpawnNode>>,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            current_camera_target_node_name: Some("start".to_owned()),
            ..Self::default()
        }
    }
}

impl Level {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_camera_target_node(
        &mut self,
        name: &str,
        position: Vec2<f32>,
        waiting_time: u128,
        next_node_name: &str,
        spawn_patterns: VecDeque<PatternNode>,
    ) {
        let next_node = if next_node_name.is_empty() {
            None
        } else {
            Some(next_node_name.to_owned())
        };

        let node = NodePoint {
            name: name.to_owned(),
            position,
            next_node,
            waiting_time,
            spawn_patterns,
        };

        self.all_nodes.insert(name.to_owned(), node);
    }

    pub fn update(&mut self) {
        if !self.current_node_spawn_patterns.is_empty() {
            while let Some(node) = self.current_node_spawn_patterns.get(0) {
                let delay = node.delay;
                if self.spawn_duration >= delay {
                    match self.current_node_spawn_patterns.pop_front() {
                        Some(pattern_node) => match self.get_pattern(&pattern_node.pattern) {
                            Some(new_list) => {
                                self.active_patterns.push(new_list);
                            }
                            None => {
                                self.is_start = false;
                            }
                        },
                        None => break,
                    };

                    self.spawn_duration -= delay;
                } else {
                    break;
                }
            }

            match self
                .spawn_duration
                .checked_add(crate::ONE_FRAME.as_millis())
            {
                Some(v) => self.spawn_duration = v,
                None => self.spawn_duration = 0,
            };
        }

        // Update active patterns
        for pattern in &mut self.active_patterns {
            // Loop for spawn enemy from pattern
            for node in pattern.iter_mut() {
                if let Some(new_delay) = node.delay.checked_sub(crate::ONE_FRAME.as_millis()) {
                    node.delay = new_delay;
                    break;
                }
                // println!("Spawn enemy: {}", node.enemy_type);
                let mut need_to_spawn_enemy_list = crate::ENEMY_SPAWN_NODES.lock().unwrap();
                need_to_spawn_enemy_list.push(node.clone());
                node.delay = 0;
            }

            // Loop for remove spawn_node from pattern
            loop {
                let result = pattern.get(0).map_or(false, |node| node.delay == 0);

                if result {
                    pattern.pop_front();
                } else {
                    break;
                }
            }
        }

        // Loop for remove empty pattern. (alreay spawned all spawn_nodes)
        while let Some(active_list) = self.active_patterns.get(0) {
            if active_list.is_empty() {
                self.active_patterns.remove(0);
            } else {
                break;
            }
        }
    }

    #[must_use]
    pub fn is_spawn_queue_empty(&self) -> bool {
        self.current_node_spawn_patterns.is_empty()
            && self.active_patterns.is_empty()
            && self.current_node_spawn_patterns.is_empty()
            && self.is_start
    }

    fn get_pattern(&self, pattern: &str) -> Option<VecDeque<EnemySpawnNode>> {
        self.patterns.get(pattern).cloned()
    }

    pub fn add_pattern(&mut self, name: &str, pattern: VecDeque<EnemySpawnNode>) {
        self.patterns.insert(name.to_owned(), pattern);
    }

    #[must_use]
    pub fn get_node(&self, key: &str) -> Option<&NodePoint> {
        self.all_nodes.get(key)
    }

    #[must_use]
    pub const fn all_nodes(&self) -> &HashMap<String, NodePoint> {
        &self.all_nodes
    }

    pub fn set_current_node(&mut self, name: &str) {
        if self.all_nodes.contains_key(name) {
            self.current_camera_target_node_name = Some(name.to_owned());
            if let Some(node) = self.get_node(name) {
                self.current_node_spawn_patterns = node.spawn_patterns.clone();
                self.is_start = true;
            };
        } else {
            self.current_camera_target_node_name = None;
        }
    }

    #[must_use]
    pub fn get_current_node(&self) -> Option<&NodePoint> {
        match self.current_camera_target_node_name.as_ref() {
            Some(name) => self.get_node(name.as_str()),
            None => self.get_node(""),
        }
    }

    #[must_use]
    pub fn get_next_node(&self) -> Option<&NodePoint> {
        match self.get_current_node() {
            Some(current_node) => match current_node.next_node.as_ref() {
                Some(next_node_name) => self.get_node(next_node_name.as_str()),
                None => None,
            },
            None => None,
        }
    }
}

#[derive(Clone)]
/// Use for fetching pattern and spawn pattern
pub struct PatternNode {
    pub delay: u128,

    pub pattern: String,
}

#[derive(Clone)]
/// Same enemy but different movement will have `enemy_type` number.
pub struct EnemySpawnNode {
    /// Waiting time before spawning enemy
    pub delay: u128,
    /// Enemy type of spawning enemy
    pub enemy_type: i32,
    /// Position of spawning enemy
    pub position: Vec2<f32>,

    pub extra: String,
}

impl EnemySpawnNode {
    #[must_use]
    pub fn new(delay: u128, enemy_type: i32, position: Vec2<f32>, extra: &str) -> Self {
        Self {
            delay,
            enemy_type,
            position,
            extra: extra.to_owned(),
        }
    }
}
