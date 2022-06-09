use crate::components::Component;
use crate::managers::details_ui::DetailsUI;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MetadataComponent {
    data: HashMap<String, String>,
}
impl MetadataComponent {
    #[allow(dead_code)]
    fn new() -> Self {
        return Self::default();
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        return self.data.get(key);
    }

    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_owned(), value.to_owned());
    }

    pub fn clear_metadata(&mut self, key: &str) {
        self.data.remove(key);
    }
}

impl Component for MetadataComponent {
    fn get_component_type() -> u64 {
        4
    }
}

impl DetailsUI for MetadataComponent {
    fn draw_details_ui(&mut self, ui: &mut egui::Ui) {
        for (key, value) in self.data.iter() {
            ui.columns(2, |cols| {
                cols[0].label(key);
                cols[1].label(value);
            });
        }
    }
}
