use crate::system::{Borrow, RunState};

pub trait SystemParameter: Send + Sync {
    type This<'this>;
    type Lock<'state>;

    fn lock(state: RunState<'_>) -> Self::Lock<'_>;
    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this>;
    fn get_resource_types() -> impl Iterator<Item = Borrow>;
    fn get_component_types() -> impl Iterator<Item = Borrow>;
}

macro_rules! system_parameter_tuple {
    ($($param:ident),*) => {
        impl<$($param),*> SystemParameter for ($($param,)*)
        where
            $($param: SystemParameter,)*
        {
            type This<'this> = ($($param::This<'this>,)*);
            type Lock<'state> = ($($param::Lock<'state>,)*);

            #[allow(clippy::unused_unit)]
            fn lock(state: RunState<'_>) -> Self::Lock<'_> {
                _ = state;
                ($($param::lock(state),)*)
            }

            #[allow(clippy::unused_unit)]
            fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
                #[allow(non_snake_case)]
                let ($($param,)*) = state;
                ($($param::construct($param),)*)
            }

            fn get_resource_types() -> impl Iterator<Item = Borrow> {
                std::iter::empty()
                    $(
                        .chain($param::get_resource_types())
                    )*
            }

            fn get_component_types() -> impl Iterator<Item = Borrow> {
                std::iter::empty()
                    $(
                        .chain($param::get_component_types())
                    )*
            }
        }
    };
}

system_parameter_tuple!();
system_parameter_tuple!(A);
system_parameter_tuple!(A, B);
system_parameter_tuple!(A, B, C);
system_parameter_tuple!(A, B, C, D);
system_parameter_tuple!(A, B, C, D, E);
system_parameter_tuple!(A, B, C, D, E, F);
system_parameter_tuple!(A, B, C, D, E, F, G);
system_parameter_tuple!(A, B, C, D, E, F, G, H);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
