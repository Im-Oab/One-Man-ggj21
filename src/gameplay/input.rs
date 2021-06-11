use tetra::input::{self, Key};
use tetra::Context;

pub trait Input {
    fn up(&self) -> bool;
    fn down(&self) -> bool;
    fn left(&self) -> bool;
    fn right(&self) -> bool;
    fn attack(&self) -> bool;
    fn switch(&self) -> bool;
    fn update(&mut self, ctx: &mut Context);
}

pub struct Keyboard {
    pub up_key: Button,
    pub down_key: Button,
    pub left_key: Button,
    pub right_key: Button,
    pub attack_key: Button,
    pub switch_key: Button,
}

impl Keyboard {
    pub fn new_with_preset_keys() -> Keyboard {
        Keyboard::new(Key::Up, Key::Down, Key::Left, Key::Right, Key::X, Key::Z)
    }

    pub fn new(
        up: Key,
        down: Key,
        left: Key,
        right: Key,
        attack_key: Key,
        switch_key: Key,
    ) -> Keyboard {
        Keyboard {
            up_key: Button::new(up),
            down_key: Button::new(down),
            left_key: Button::new(left),
            right_key: Button::new(right),
            attack_key: Button::new(attack_key),
            switch_key: Button::new(switch_key),
        }
    }
}

impl Input for Keyboard {
    fn up(&self) -> bool {
        self.up_key.hold_time > 0
    }

    fn down(&self) -> bool {
        self.down_key.hold_time > 0
    }

    fn left(&self) -> bool {
        self.left_key.hold_time > 0
    }

    fn right(&self) -> bool {
        self.right_key.hold_time > 0
    }

    fn attack(&self) -> bool {
        self.attack_key.hold_time < 50 && self.attack_key.idle_time == 0
    }

    fn switch(&self) -> bool {
        self.switch_key.hold_time < 50 && self.switch_key.idle_time == 0
    }

    fn update(&mut self, ctx: &mut Context) {
        self.up_key.update(ctx);
        self.down_key.update(ctx);
        self.left_key.update(ctx);
        self.right_key.update(ctx);
        self.attack_key.update(ctx);
        self.switch_key.update(ctx);
    }
}

pub struct Button {
    button: Key,
    pub hold_time: u128,
    pub idle_time: u128,
}

impl Button {
    pub fn new(button: Key) -> Button {
        Button {
            button: button,
            idle_time: 200,
            hold_time: 0,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let delta_time = 1000 / 60;
        self.idle_time += delta_time;
        if input::is_key_down(ctx, self.button) {
            if self.hold_time == 0 {
                self.idle_time = 0;
            }
            self.hold_time += delta_time;
        } else {
            self.hold_time = 0;
        }
    }

    pub fn consume(&mut self) {
        self.idle_time = 200;
    }
}
