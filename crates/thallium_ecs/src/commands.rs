use crate::{
    component::ComponentBundle,
    system::{CommandSender, SystemRunState},
    App, Entity, SystemParameter,
};

/// A [`SystemParameter`] that allows you to create/destroy [`Entity`]s, add/remove [`Component`](crate::Component)s, etc
pub struct Commands<'a> {
    command_sender: &'a CommandSender,
}

impl Commands<'_> {
    /// Creates an [`Entity`]
    pub fn create_entity<B>(&mut self, bundle: B)
    where
        B: ComponentBundle,
    {
        self.command_sender
            .send(Box::new(move |app| {
                let entity = app.create_entity();
                bundle.add(app, entity);
            }))
            .unwrap();
    }

    /// Schedules an [`Entity`] to be destroyed along with all its attached components
    /// This does not error if the [`Entity`] is already destroyed
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.command_sender
            .send(Box::new(move |app| app.destroy_entity(entity)))
            .unwrap();
    }

    /// Adds a bundle of [`Component`](crate::Component)s to an [`Entity`], any [`Component`](crate::Component)s that are already attached will be replaced
    /// This does not error if the [`Entity`] is invalid/destroyed
    pub fn add_components<B>(&mut self, entity: Entity, bundle: B)
    where
        B: ComponentBundle,
    {
        self.command_sender
            .send(Box::new(move |app| bundle.add(app, entity)))
            .unwrap();
    }

    /// Removes a bundle of [`Component`](crate::Component)s from an [`Entity`]
    /// This does not error if the [`Entity`] is invalid/destroyed or any of the [`Component`](crate::Component)s are not attached to this [`Entity`]
    pub fn remove_components<B>(&mut self, entity: Entity)
    where
        B: ComponentBundle,
    {
        self.command_sender
            .send(Box::new(move |app| B::remove(app, entity)))
            .unwrap();
    }

    /// Schedules an arbitrary closure to be run after the current [`SystemSet`](crate::SystemSet) has finished
    pub fn schedule(&mut self, f: impl FnOnce(&mut App) + Send + 'static) {
        self.command_sender
            .send(Box::new(move |app| f(app)))
            .unwrap();
    }
}

impl<'a> SystemParameter for Commands<'a> {
    type This<'this> = Commands<'this>;
    type Lock<'state> = &'state CommandSender;

    fn lock<'state>(state: &SystemRunState<'state>) -> Self::Lock<'state> {
        state.command_sender
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>, last_run_tick: u64) -> Self::This<'this> {
        _ = last_run_tick;
        Commands {
            command_sender: state,
        }
    }

    fn get_resource_types() -> impl Iterator<Item = crate::system::Borrow> {
        std::iter::empty()
    }

    fn get_component_types() -> impl Iterator<Item = crate::system::Borrow> {
        std::iter::empty()
    }
}
