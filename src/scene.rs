use tetra::Context;

/// Every scene have to implement this trait.
pub trait Scene {
    /// Every update in the scene go in here
    /// Return Transition::None when scene still active and
    /// other Transitions for changing scene.
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition>;
    /// Every draw in the scene go in here.
    fn draw(&mut self, ctx: &mut Context);
}

/// Scene transition,
/// None: Dont change scene.
/// Pop: Pop current scene. Previous scene will become active.
/// Push: Add new scene on top of the stack. Current scene will become inactive.
/// Replace: Clear scene stack and Add new scene.
pub enum Transition {
    None,
    Pop,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
}
