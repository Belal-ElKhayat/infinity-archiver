use eframe::egui::{self, Color32, FontId, Margin, Rounding, Stroke, TextStyle};

pub fn apply_fluent_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (TextStyle::Heading, FontId::proportional(20.0)),
        (TextStyle::Body, FontId::proportional(14.0)),
        (TextStyle::Monospace, FontId::monospace(13.0)),
        (TextStyle::Button, FontId::proportional(14.0)),
        (TextStyle::Small, FontId::proportional(12.0)),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = Margin::same(12.0);

    let rounding = Rounding::same(8.0);
    style.visuals.widgets.noninteractive.rounding = rounding;
    style.visuals.widgets.inactive.rounding = rounding;
    style.visuals.widgets.hovered.rounding = rounding;
    style.visuals.widgets.active.rounding = rounding;

    if dark_mode {
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(20, 20, 27);
        style.visuals.window_fill = Color32::from_rgb(24, 24, 33);
        style.visuals.faint_bg_color = Color32::from_rgb(30, 30, 42);
        style.visuals.selection.bg_fill = Color32::from_rgb(124, 77, 255);
        style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(255, 215, 64));
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(34, 34, 48);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 46, 70);
    } else {
        style.visuals = egui::Visuals::light();
        style.visuals.panel_fill = Color32::from_rgb(245, 246, 250);
        style.visuals.selection.bg_fill = Color32::from_rgb(103, 58, 183);
    }

    ctx.set_style(style);
}
