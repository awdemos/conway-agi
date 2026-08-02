use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub background: Color,
    pub panel_bg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub text: Color,
    pub muted: Color,
    pub dim: Color,
    pub warning: Color,
    pub genome_colors: [Color; 16],
    pub signal_colors: [Color; 5],
    pub energy_glyphs: [char; 6],
    pub voxel_glyphs: [char; 8],
    pub bloom_glyphs: [char; 3],
    pub border_heavy: BorderGlyphs,
    pub border_double: BorderGlyphs,
}

#[derive(Clone, Copy, Debug)]
pub struct BorderGlyphs {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

impl Theme {
    pub fn neon() -> Self {
        Self {
            background: Color::Rgb(5, 5, 5),
            panel_bg: Color::Rgb(10, 10, 16),
            primary: Color::Rgb(246, 0, 86),
            secondary: Color::Rgb(0, 240, 255),
            tertiary: Color::Rgb(57, 255, 20),
            text: Color::Rgb(224, 224, 224),
            muted: Color::Rgb(122, 122, 154),
            dim: Color::Rgb(58, 58, 80),
            warning: Color::Rgb(255, 158, 0),
            genome_colors: [
                Color::Rgb(246, 0, 86),
                Color::Rgb(255, 158, 0),
                Color::Rgb(57, 255, 20),
                Color::Rgb(0, 240, 255),
                Color::Rgb(119, 10, 127),
                Color::Rgb(170, 0, 0),
                Color::Rgb(73, 0, 184),
                Color::Rgb(255, 0, 128),
                Color::Rgb(0, 255, 170),
                Color::Rgb(201, 0, 255),
                Color::Rgb(255, 255, 0),
                Color::Rgb(0, 128, 255),
                Color::Rgb(255, 20, 147),
                Color::Rgb(0, 255, 128),
                Color::Rgb(255, 69, 0),
                Color::Rgb(138, 43, 226),
            ],
            signal_colors: [
                Color::Rgb(58, 58, 80),
                Color::Rgb(255, 255, 120),
                Color::Rgb(0, 128, 255),
                Color::Rgb(0, 240, 255),
                Color::Rgb(180, 255, 255),
            ],
            energy_glyphs: ['∙', '·', '░', '▒', '▓', '█'],
            voxel_glyphs: ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'],
            bloom_glyphs: ['░', '▒', '▓'],
            border_heavy: BorderGlyphs {
                top_left: '┏',
                top_right: '┓',
                bottom_left: '┗',
                bottom_right: '┛',
                horizontal: '━',
                vertical: '┃',
            },
            border_double: BorderGlyphs {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '═',
                vertical: '║',
            },
        }
    }

    pub fn voxel() -> Self {
        let mut theme = Self::neon();
        theme.background = Color::Rgb(2, 2, 8);
        theme.panel_bg = Color::Rgb(5, 5, 16);
        theme.dim = Color::Rgb(30, 30, 60);
        theme
    }

    pub fn crt() -> Self {
        Self {
            background: Color::Rgb(0, 17, 0),
            panel_bg: Color::Rgb(0, 25, 0),
            primary: Color::Rgb(0, 255, 65),
            secondary: Color::Rgb(0, 255, 255),
            tertiary: Color::Rgb(255, 176, 0),
            text: Color::Rgb(0, 255, 65),
            muted: Color::Rgb(0, 143, 17),
            dim: Color::Rgb(0, 80, 10),
            warning: Color::Rgb(255, 176, 0),
            genome_colors: [
                Color::Rgb(0, 255, 65),
                Color::Rgb(0, 240, 120),
                Color::Rgb(0, 255, 170),
                Color::Rgb(0, 255, 200),
                Color::Rgb(0, 255, 255),
                Color::Rgb(120, 255, 120),
                Color::Rgb(180, 255, 180),
                Color::Rgb(255, 255, 255),
                Color::Rgb(0, 200, 80),
                Color::Rgb(0, 180, 100),
                Color::Rgb(0, 160, 120),
                Color::Rgb(0, 140, 140),
                Color::Rgb(0, 120, 160),
                Color::Rgb(0, 100, 180),
                Color::Rgb(0, 80, 200),
                Color::Rgb(0, 60, 220),
            ],
            signal_colors: [
                Color::Rgb(0, 40, 0),
                Color::Rgb(0, 100, 0),
                Color::Rgb(0, 160, 0),
                Color::Rgb(0, 220, 0),
                Color::Rgb(120, 255, 120),
            ],
            energy_glyphs: ['·', '∙', '░', '▒', '▓', '█'],
            voxel_glyphs: ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'],
            bloom_glyphs: ['░', '▒', '▓'],
            border_heavy: BorderGlyphs {
                top_left: '┏',
                top_right: '┓',
                bottom_left: '┗',
                bottom_right: '┛',
                horizontal: '━',
                vertical: '┃',
            },
            border_double: BorderGlyphs {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '═',
                vertical: '║',
            },
        }
    }

    pub const fn genome_color(&self, index: u8) -> Color {
        self.genome_colors[index as usize % self.genome_colors.len()]
    }

    pub fn signal_color(&self, value: u8) -> Color {
        let idx = match value {
            0..=51 => 0,
            52..=103 => 1,
            104..=155 => 2,
            156..=207 => 3,
            _ => 4,
        };
        self.signal_colors[idx]
    }

    pub fn energy_glyph(&self, energy: u8) -> char {
        let idx = match energy {
            230..=255 => 5,
            180..=229 => 4,
            130..=179 => 3,
            80..=129 => 2,
            30..=79 => 1,
            _ => 0,
        };
        self.energy_glyphs[idx]
    }

    pub fn voxel_glyph(&self, energy: u8) -> char {
        let idx = match energy {
            224..=255 => 7,
            192..=223 => 6,
            160..=191 => 5,
            128..=159 => 4,
            96..=127 => 3,
            64..=95 => 2,
            32..=63 => 1,
            _ => 0,
        };
        self.voxel_glyphs[idx]
    }

    pub fn bloom_glyph(&self, distance: u8) -> char {
        let idx = match distance {
            0..=1 => 2,
            2..=3 => 1,
            _ => 0,
        };
        self.bloom_glyphs[idx]
    }
}
