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
        self.base()
    }
    fn diff_gutter_bg(&self) -> Color {
        self.surface0()
    }
    fn annotation_text(&self) -> Color {
        self.subtext1()
    }
    fn inactive_border(&self) -> Color {
        self.overlay0()
    }
    fn focused_border(&self) -> Color {
        self.overlay2()
    }
}
