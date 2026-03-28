use unicode_segmentation::UnicodeSegmentation;

/// Off-screen 2D character canvas using grapheme clusters.
/// `cells[row][col]` holds a single grapheme (default space).
pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Vec<String>>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Canvas {
            width,
            height,
            cells: vec![vec![String::from(" "); width]; height],
        }
    }

    /// Place a single grapheme at (col, row). Out-of-bounds writes are silently ignored.
    pub fn set(&mut self, col: usize, row: usize, ch: &str) {
        if row < self.height && col < self.width {
            self.cells[row][col] = ch.to_string();
        }
    }

    /// Draw text left-to-right starting at (col, row). Clips at canvas edge.
    pub fn draw_text(&mut self, col: usize, row: usize, text: &str) {
        for (i, g) in text.graphemes(true).enumerate() {
            self.set(col + i, row, g);
        }
    }

    /// Draw a horizontal line of `ch` with given length starting at (col, row).
    pub fn hline(&mut self, col: usize, row: usize, len: usize, ch: char) {
        let s = ch.to_string();
        for i in 0..len {
            self.set(col + i, row, &s);
        }
    }

    /// Draw a vertical line of `ch` with given length starting at (col, row).
    pub fn vline(&mut self, col: usize, row: usize, len: usize, ch: char) {
        let s = ch.to_string();
        for i in 0..len {
            self.set(col, row + i, &s);
        }
    }

    /// Render to lines of text, trimming trailing spaces from each row.
    pub fn render(&self) -> Vec<String> {
        self.cells
            .iter()
            .map(|row| {
                let s: String = row.concat();
                s.trim_end().to_string()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_set_and_render() {
        let mut c = Canvas::new(5, 3);
        c.set(0, 0, "+");
        c.set(4, 0, "+");
        c.set(0, 2, "+");
        c.set(4, 2, "+");
        let lines = c.render();
        assert_eq!(lines[0], "+   +");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "+   +");
    }

    #[test]
    fn test_canvas_hline_vline() {
        let mut c = Canvas::new(5, 5);
        c.hline(0, 0, 5, '-');
        c.vline(0, 0, 5, '|');
        let lines = c.render();
        assert_eq!(lines[0], "|----");
        assert_eq!(lines[1], "|");
        assert_eq!(lines[4], "|");
    }

}
