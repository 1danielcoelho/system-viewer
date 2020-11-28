use crate::managers::ComponentManager;

use super::{component::ComponentIndex, Component};

#[derive(Clone)]
pub enum WidgetType {
    None,
    TestWidget,
}

#[derive(Clone)]
pub struct UIComponent {
    enabled: bool,
    pub widget_type: WidgetType,
}
impl UIComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for UIComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            widget_type: WidgetType::None,
        };
    }
}
impl Component for UIComponent {
    type ComponentType = UIComponent;

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Ui;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<Self::ComponentType> {
        return &mut w.interface;
    }
}
