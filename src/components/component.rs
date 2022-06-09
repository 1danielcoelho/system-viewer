use crate::managers::details_ui::DetailsUI;

pub trait Component: Default + Clone + DetailsUI {
    fn get_component_type() -> u64;
}
