use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use tetra::math::Vec2;
use tetra::Context;

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

impl Level {
    pub fn new() -> Level {
        Level {
            is_start: false,
            backgrounds: vec![],
            current_camera_target_node_name: Some(String::from("start")),
            all_nodes: HashMap::new(),
            patterns: HashMap::new(),
            spawn_duration: 0,
            current_node_spawn_patterns: VecDeque::new(),
            active_patterns: vec![],
        }
    }

    pub fn add_camera_target_node(
        &mut self,
        name: &str,
        position: Vec2<f32>,
        waiting_time: u128,
        next_node_name: &str,
        spawn_patterns: VecDeque<PatternNode>,
    ) {
        let next_node = match next_node_name.len() {
            0 => None,
            _ => Some(String::from(next_node_name)),
        };

        let node = NodePoint {
            name: String::from(name),
            position: position,
            next_node: next_node,
            waiting_time: waiting_time,
            spawn_patterns: spawn_patterns,
        };

        self.all_nodes.insert(String::from(name), node);
    }

    pub fn update(&mut self) {
        if self.current_node_spawn_patterns.len() > 0 {
            loop {
                let some_node = self.current_node_spawn_patterns.get(0);
                if some_node.is_some() {
                    let node = some_node.unwrap();
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
        for pattern in self.active_patterns.iter_mut() {
            // Loop for spawn enemy from pattern
            for node in pattern.iter_mut() {
                match node.delay.checked_sub(crate::ONE_FRAME.as_millis()) {
                    Some(new_delay) => {
                        node.delay = new_delay;
                        break;
                    }
                    None => {
                        // println!("Spawn enemy: {}", node.enemy_type);
                        let mut need_to_spawn_enemy_list = crate::ENEMY_SPAWN_NODES.lock().unwrap();
                        need_to_spawn_enemy_list.push(node.clone());
                        node.delay = 0;
                    }
                }
            }

            // Loop for remove spawn_node from pattern
            loop {
                let result = match pattern.get(0) {
                    Some(node) => node.delay == 0,
                    None => false,
                };

                if result {
                    pattern.pop_front();
                } else {
                    break;
                }
            }
        }

        // Loop for remove empty pattern. (alreay spawned all spawn_nodes)
        loop {
            let result = match self.active_patterns.get(0) {
                Some(active_list) => active_list.len() == 0,
                None => break,
            };

            if result {
                self.active_patterns.remove(0);
            } else {
                break;
            }
        }
    }

    pub fn is_spawn_queue_empty(&self) -> bool {
        self.current_node_spawn_patterns.len() == 0
            && self.active_patterns.len() == 0
            && self.current_node_spawn_patterns.len() == 0
            && self.is_start == true
    }

    fn get_pattern(&self, pattern: &String) -> Option<VecDeque<EnemySpawnNode>> {
        match self.patterns.get(pattern) {
            Some(list) => {
                return Some(list.clone());
            }
            None => (),
        };

        None
    }

    pub fn add_pattern(&mut self, name: &str, pattern: VecDeque<EnemySpawnNode>) {
        self.patterns.insert(String::from(name), pattern);
    }

    pub fn get_node(&self, key: &str) -> Option<&NodePoint> {
        self.all_nodes.get(key)
    }

    pub fn all_nodes(&self) -> &HashMap<String, NodePoint> {
        &self.all_nodes
    }

    pub fn set_current_node(&mut self, name: &str) {
        if self.all_nodes.contains_key(name) {
            self.current_camera_target_node_name = Some(String::from(name));
            match self.get_node(name) {
                Some(node) => {
                    self.current_node_spawn_patterns = node.spawn_patterns.clone();
                    self.is_start = true;
                }
                None => (),
            };
        } else {
            self.current_camera_target_node_name = None;
        }
    }

    pub fn get_current_node(&self) -> Option<&NodePoint> {
        match self.current_camera_target_node_name.as_ref() {
            Some(name) => self.get_node(name.as_str()),
            None => self.get_node(""),
        }
    }

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
/// Same enemy but different movement will have enemy_type number.
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
    pub fn new(delay: u128, enemy_type: i32, position: Vec2<f32>, extra: &str) -> EnemySpawnNode {
        EnemySpawnNode {
            delay: delay,
            enemy_type: enemy_type,
            position: position,
            extra: String::from(extra),
        }
    }
}
