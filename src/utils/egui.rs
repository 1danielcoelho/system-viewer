pub fn is_position_over_egui(pos: egui::Pos2, ctx: &egui::Context) -> bool {
    if let Some(layer) = ctx.layer_id_at(pos) {
        if layer.order == egui::Order::Background {
            return !ctx.available_rect().contains(pos);
        } else {
            true
        }
    } else {
        false
    }
}
