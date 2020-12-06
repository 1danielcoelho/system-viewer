use std::collections::HashMap;

use crate::managers::{ECManager, Entity};

use super::{
    component::{ComponentIndex, ComponentStorageType},
    Component,
};

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
    const STORAGE_TYPE: ComponentStorageType = ComponentStorageType::HashMap;

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Ui;
    }

    fn get_components_map<'a>(
        w: &'a mut ECManager,
    ) -> Option<&'a mut HashMap<Entity, UIComponent>> {
        return Some(&mut w.interface);
    }
}
