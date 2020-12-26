use egui::Ui;

pub trait DetailsUI {
    fn draw_details_ui(&mut self, _ui: &mut Ui) {}
}
