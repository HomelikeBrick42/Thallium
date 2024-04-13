use crate::SystemParameter;

/// A [`SystemParameter`] for getting the current tick
pub struct CurrentTick(pub u64);

impl SystemParameter for CurrentTick {
    type This<'this> = CurrentTick;
    type Lock<'state> = u64;

    fn lock<'state>(state: &crate::system::SystemRunState<'state>) -> Self::Lock<'state> {
        state.current_tick
    }

    fn construct<'this>(
        state: &'this mut Self::Lock<'_>,
        _last_run_tick: u64,
    ) -> Self::This<'this> {
        CurrentTick(*state)
    }

    fn get_resource_types() -> impl Iterator<Item = crate::system::Borrow> {
        std::iter::empty()
    }

    fn get_component_types() -> impl Iterator<Item = crate::system::Borrow> {
        std::iter::empty()
    }
}

/// A [`SystemParameter`] for getting the last tick that the current [`System`](crate::System) was run
///
/// Be aware that [`App::run`](crate::App::run) takes [`IntoSystem`](crate::IntoSystem),
/// so if you pass it a closure that means a new [`System`](crate::System) is being made every time you call [`App::run`](crate::App::run)
pub struct LastRunTick(pub u64);

impl SystemParameter for LastRunTick {
    type This<'this> = LastRunTick;
    type Lock<'state> = ();

    fn lock<'state>(_state: &crate::system::SystemRunState<'state>) -> Self::Lock<'state> {}

    fn construct<'this>(
        _state: &'this mut Self::Lock<'_>,
        last_run_tick: u64,
    ) -> Self::This<'this> {
        LastRunTick(last_run_tick)
    }

    fn get_resource_types() -> impl Iterator<Item = crate::system::Borrow> {
        std::iter::empty()
    }

    fn get_component_types() -> impl Iterator<Item = crate::system::Borrow> {
        std::iter::empty()
    }
}
