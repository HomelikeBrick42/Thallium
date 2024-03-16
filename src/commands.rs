use crate::{
    component::ComponentBundle,
    entities::Entity,
    system::{CommandSender, RunState},
    system_parameters::SystemParameter,
};

pub struct Commands<'a> {
    command_sender: &'a CommandSender,
}

impl Commands<'_> {
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

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.command_sender
            .send(Box::new(move |app| app.destroy_entity(entity)))
            .unwrap();
    }

    pub fn add_components<B>(&mut self, entity: Entity, bundle: B)
    where
        B: ComponentBundle,
    {
        self.command_sender
            .send(Box::new(move |app| bundle.add(app, entity)))
            .unwrap();
    }

    pub fn remove_components<B>(&mut self, entity: Entity)
    where
        B: ComponentBundle,
    {
        self.command_sender
            .send(Box::new(move |app| B::remove(app, entity)))
            .unwrap();
    }
}

impl<'a> SystemParameter for Commands<'a> {
    type This<'this> = Commands<'this>;
    type Lock<'state> = &'state CommandSender;

    fn lock(state: RunState<'_>) -> Self::Lock<'_> {
        state.command_sender
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
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