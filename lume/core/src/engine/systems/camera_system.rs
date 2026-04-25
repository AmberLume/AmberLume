use shipyard::{IntoIter, UniqueViewMut, View};
use amber_lume::world::components::position_component::PositionComponent;
use amber_lume::world::components::user_controllable_component::UserControllableComponent;
use amber_lume::world::unique::render_view_unique::RenderViewUnique;

pub fn camera_system(
    positions: View<PositionComponent>,
    user_controllable_component: View<UserControllableComponent>,
    mut render_view_unique: UniqueViewMut<RenderViewUnique>,
) {
    for (position, _user_controllable) in (&positions, &user_controllable_component).iter() {
        render_view_unique.camera_view.target = position.position;
    }
}
