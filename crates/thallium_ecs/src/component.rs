use crate::{App, Entity};

pub trait Component: Sized + Send + Sync + 'static {}

pub trait ComponentBundle: Sized + Send + Sync + 'static {
    fn add(self, app: &mut App, entity: Entity);
    fn remove(app: &mut App, entity: Entity);
}

impl<C> ComponentBundle for C
where
    C: Component,
{
    fn add(self, app: &mut App, entity: Entity) {
        app.add_component(entity, self);
    }

    fn remove(app: &mut App, entity: Entity) {
        app.remove_component::<C>(entity);
    }
}

macro_rules! component_bundle_tuple {
    ($($param:ident),*) => {
        impl<$($param),*> ComponentBundle for ($($param,)*)
        where
            $($param: ComponentBundle,)*
        {
            fn add(self, app: &mut App, entity: Entity) {
                _ = app;
                _ = entity;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                $(
                    $param.add(app, entity);
                )*
            }

            fn remove(app: &mut App, entity: Entity) {
                _ = app;
                _ = entity;
                $(
                    $param::remove(app, entity);
                )*
            }
        }
    };
}

component_bundle_tuple!();
component_bundle_tuple!(A);
component_bundle_tuple!(A, B);
component_bundle_tuple!(A, B, C);
component_bundle_tuple!(A, B, C, D);
component_bundle_tuple!(A, B, C, D, E);
component_bundle_tuple!(A, B, C, D, E, F);
component_bundle_tuple!(A, B, C, D, E, F, G);
component_bundle_tuple!(A, B, C, D, E, F, G, H);
component_bundle_tuple!(A, B, C, D, E, F, G, H, I);
component_bundle_tuple!(A, B, C, D, E, F, G, H, I, J);
component_bundle_tuple!(A, B, C, D, E, F, G, H, I, J, K);
component_bundle_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
component_bundle_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
component_bundle_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
component_bundle_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
component_bundle_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
