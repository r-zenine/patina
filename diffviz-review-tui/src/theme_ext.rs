use ratatui::style::Color;
use tui_design::SurfaceRamp;

pub trait DiffvizSurface {
    fn panel_bg(&self) -> Color;
    fn diff_gutter_bg(&self) -> Color;
    fn annotation_text(&self) -> Color;
    fn inactive_border(&self) -> Color;
    fn focused_border(&self) -> Color;
}

impl DiffvizSurface for SurfaceRamp {
    fn panel_bg(&self) -> Color {
        self[0]
    }
    fn diff_gutter_bg(&self) -> Color {
        self[2]
    }
    fn annotation_text(&self) -> Color {
        self[6]
    }
    fn inactive_border(&self) -> Color {
        self[3]
    }
    fn focused_border(&self) -> Color {
        self[4]
    }
}
