use std::ops::Index;

use ratatui::style::Color;

use crate::palette;

/// Eleven-step luminance ramp from darkest (0) to brightest (10).
///
/// Index → Catppuccin Mocha name:
///   0 = crust, 1 = mantle, 2 = base, 3 = surface0, 4 = surface1,
///   5 = surface2, 6 = overlay0, 7 = overlay2, 8 = subtext0,
///   9 = subtext1, 10 = text
///
/// Structural elevation only uses crust..=surface1 (see `CardTier` and the
/// `stylesheet` layer functions). `surface2` is reserved for selection state
/// so an in-flow selection can never collide with structural elevation.
#[derive(Clone, Copy)]
pub struct SurfaceRamp(pub [Color; 11]);

impl Index<usize> for SurfaceRamp {
    type Output = Color;
    fn index(&self, i: usize) -> &Color {
        &self.0[i]
    }
}

impl SurfaceRamp {
    pub fn crust(&self) -> Color {
        self.0[0]
    }
    pub fn mantle(&self) -> Color {
        self.0[1]
    }
    pub fn base(&self) -> Color {
        self.0[2]
    }
    pub fn surface0(&self) -> Color {
        self.0[3]
    }
    pub fn surface1(&self) -> Color {
        self.0[4]
    }
    /// Reserved for selection state (`stylesheet::selection`) — never
    /// use as a structural tier background.
    pub fn surface2(&self) -> Color {
        self.0[5]
    }
    pub fn overlay0(&self) -> Color {
        self.0[6]
    }
    pub fn overlay2(&self) -> Color {
        self.0[7]
    }
    pub fn subtext0(&self) -> Color {
        self.0[8]
    }
    pub fn subtext1(&self) -> Color {
        self.0[9]
    }
    pub fn text(&self) -> Color {
        self.0[10]
    }
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
                c.crust.into(),
                c.mantle.into(),
                c.base.into(),
                c.surface0.into(),
                c.surface1.into(),
                c.surface2.into(),
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
