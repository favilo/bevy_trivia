use bevy::prelude::*;
use bevy_quill::Cx;

pub mod dropdown;
pub mod multi_dropdown;
pub mod text_input;

pub trait UseComponentOrDefault {
    fn use_component_or_default<T: Component + Default>(&mut self, target: Entity) -> &T;
    fn use_component_or<T: Component>(&mut self, target: Entity, default: T) -> &T;
}

impl<'p, 'w> UseComponentOrDefault for Cx<'p, 'w> {
    fn use_component_or_default<C: Component + Default>(&mut self, target: Entity) -> &C {
        self.use_component_or(target, C::default())
    }

    fn use_component_or<C: Component>(&mut self, target: Entity, default: C) -> &C {
        let mut ent = self.world_mut().entity_mut(target);
        if !ent.contains::<C>() {
            ent.insert(default);
        }
        self.use_component::<C>(target).unwrap()
    }
}
