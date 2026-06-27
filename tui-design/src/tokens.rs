use std::ops::Index;

use ratatui::style::Color;

use crate::palette;

/// Nine-step luminance ramp from darkest (0) to brightest (8).
///
/// Index → Catppuccin Mocha name:
///   0 = base, 1 = mantle, 2 = surface0, 3 = surface1,
///   4 = overlay0, 5 = overlay2, 6 = subtext0, 7 = subtext1, 8 = text
#[derive(Clone, Copy)]
pub struct SurfaceRamp(pub [Color; 9]);

impl Index<usize> for SurfaceRamp {
    type Output = Color;
    fn index(&self, i: usize) -> &Color {
        &self.0[i]
    }
}

impl SurfaceRamp {
    pub fn base(&self) -> Color { self.0[0] }
    pub fn mantle(&self) -> Color { self.0[1] }
    pub fn surface0(&self) -> Color { self.0[2] }
    pub fn surface1(&self) -> Color { self.0[3] }
    pub fn overlay0(&self) -> Color { self.0[4] }
    pub fn overlay2(&self) -> Color { self.0[5] }
    pub fn subtext0(&self) -> Color { self.0[6] }
    pub fn subtext1(&self) -> Color { self.0[7] }
    pub fn text(&self) -> Color { self.0[8] }
}

/// All 14 Catppuccin accent colors by their catppuccin names.
/// No semantic renaming — TUIs apply meaning via extension traits.
#[derive(Clone, Copy)]
pub struct AccentPalette {
    pub rosewater: Color,
    pub flamingo: Color,
    pub pink: Color,
    pub mauve: Color,
    pub red: Color,
    pub maroon: Color,
    pub peach: Color,
    pub yellow: Color,
    pub green: Color,
    pub teal: Color,
    pub sky: Color,
    pub sapphire: Color,
    pub blue: Color,
    pub lavender: Color,
}

/// Top-level theme value: surface ramp + accent palette.
#[derive(Clone, Copy)]
pub struct Theme {
    pub surface: SurfaceRamp,
    pub accents: AccentPalette,
}

impl Theme {
    pub fn mocha() -> Self {
        let c = &palette::mocha().colors;
        Self {
            surface: SurfaceRamp([
                c.base.into(),
                c.mantle.into(),
                c.surface0.into(),
                c.surface1.into(),
                c.overlay0.into(),
                c.overlay2.into(),
                c.subtext0.into(),
                c.subtext1.into(),
                c.text.into(),
            ]),
            accents: AccentPalette {
                rosewater: c.rosewater.into(),
                flamingo: c.flamingo.into(),
                pink: c.pink.into(),
                mauve: c.mauve.into(),
                red: c.red.into(),
                maroon: c.maroon.into(),
                peach: c.peach.into(),
                yellow: c.yellow.into(),
                green: c.green.into(),
                teal: c.teal.into(),
                sky: c.sky.into(),
                sapphire: c.sapphire.into(),
                blue: c.blue.into(),
                lavender: c.lavender.into(),
            },
        }
    }
}
