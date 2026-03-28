use crate::canvas::Canvas;
use crate::cli::{AltNames, Direction, PinGap, Side};
use crate::pin::PinName;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use toml::map::Map;
use toml::Value;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QuadStyle {
    Plcc,
    Qfp,
    Bqfp,
}

/// Which corner/edge has pin 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pin1Pos {
    TopCenter,
    LeftTop,
    BottomLeft,
}

/// Quad flat package (PLCC, QFP, BQFP).
///
/// Pin numbering is counter-clockwise from above, starting at pin 1.
/// `top` = pins per top/bottom edge, `side` = pins per left/right edge.
#[derive(Debug, PartialEq)]
pub struct Quad {
    pub name: String,
    pub title: String,
    pub style: QuadStyle,
    pub top: usize,
    pub side: usize,
    pub pin1: Pin1Pos,
    pins: BTreeMap<usize, PinName>,
}

impl Quad {
    fn total(&self) -> usize {
        2 * self.top + 2 * self.side
    }

    fn pin(&self, n: usize) -> &PinName {
        self.pins.get(&n).unwrap()
    }

    /// Pin numbers for each side in CCW order starting from pin 1.
    /// Returns (top_pins, right_pins, bottom_pins, left_pins).
    /// Each vec: left→right for top/bottom, top→bottom for left/right.
    fn pin_layout(&self) -> (Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>) {
        let total = self.total();
        let t = self.top;
        let s = self.side;

        match self.pin1 {
            Pin1Pos::TopCenter => {
                let pin1_pos = t.div_ceil(2);
                let mut top = vec![0usize; t];
                let mut left = vec![0usize; s];
                let mut bottom = vec![0usize; t];
                let mut right = vec![0usize; s];
                let mut p = 1usize;
                top[..pin1_pos].iter_mut().rev().for_each(|slot| { *slot = p; p += 1; });
                left.iter_mut().for_each(|slot| { *slot = p; p += 1; });
                bottom.iter_mut().for_each(|slot| { *slot = p; p += 1; });
                right.iter_mut().rev().for_each(|slot| { *slot = p; p += 1; });
                top[pin1_pos..].iter_mut().rev().for_each(|slot| { *slot = p; p += 1; });
                assert_eq!(p - 1, total);
                (top, right, bottom, left)
            }
            Pin1Pos::LeftTop => {
                let mut left = vec![0usize; s];
                let mut bottom = vec![0usize; t];
                let mut right = vec![0usize; s];
                let mut top = vec![0usize; t];
                let mut p = 1usize;
                left.iter_mut().for_each(|slot| { *slot = p; p += 1; });
                bottom.iter_mut().for_each(|slot| { *slot = p; p += 1; });
                right.iter_mut().rev().for_each(|slot| { *slot = p; p += 1; });
                top.iter_mut().rev().for_each(|slot| { *slot = p; p += 1; });
                assert_eq!(p - 1, total);
                (top, right, bottom, left)
            }
            Pin1Pos::BottomLeft => {
                let mut bottom = vec![0usize; t];
                let mut right = vec![0usize; s];
                let mut top = vec![0usize; t];
                let mut left = vec![0usize; s];
                let mut p = 1usize;
                bottom.iter_mut().for_each(|slot| { *slot = p; p += 1; });
                right.iter_mut().rev().for_each(|slot| { *slot = p; p += 1; });
                top.iter_mut().rev().for_each(|slot| { *slot = p; p += 1; });
                left.iter_mut().for_each(|slot| { *slot = p; p += 1; });
                assert_eq!(p - 1, total);
                (top, right, bottom, left)
            }
        }
    }

    pub fn print(
        &self,
        dir: Direction,
        side: Side,
        show_pin: PinGap,
        show_alt: AltNames,
    ) -> Vec<String> {
        // Step 1: North/Top canonical layout
        let (top, right, bottom, left) = self.pin_layout();

        // Step 2: rotate pin assignment for direction (no canvas rotation)
        let (top, right, bottom, left) = match dir {
            Direction::North => (top, right, bottom, left),
            // 90° CW: new_top=left.rev, new_right=top, new_bottom=right.rev, new_left=bottom
            Direction::East => (rev(&left), top, rev(&right), bottom),
            // 180°
            Direction::South => (rev(&bottom), rev(&left), rev(&top), rev(&right)),
            // 90° CCW: new_top=right, new_right=bottom.rev, new_bottom=left, new_left=top.rev
            Direction::West => (right, rev(&bottom), left, rev(&top)),
        };

        // Step 3: Side::Bottom = left-right mirror of the Top drawing.
        // Horizontal edges (top/bottom) are reversed; vertical edges (left/right) swap, no reversal.
        let mirrored = side == Side::Bottom;
        let (top, right, bottom, left) = match side {
            Side::Top => (top, right, bottom, left),
            Side::Bottom => (rev(&top), left, rev(&bottom), right),
        };

        // Step 4: draw directly
        self.render(&top, &right, &bottom, &left, show_pin, show_alt, dir, mirrored).render()
    }

    fn render(
        &self,
        top_pins: &[usize],
        right_pins: &[usize],
        bottom_pins: &[usize],
        left_pins: &[usize],
        show_pin: PinGap,
        show_alt: AltNames,
        dir: Direction,
        mirrored: bool,
    ) -> Canvas {
        let has_pins = show_pin != PinGap::None;
        let pin_gap: usize = if has_pins { 1 } else { 0 };
        let pin_sep = "  "; // 2 spaces between horizontal name and pin number

        // Each top/bottom pin occupies exactly 2 columns (1 grapheme + 1 gap), same as DIP.
        let tb_col_w: usize = 2;

        // Row count above/below body = max primary name grapheme count for top/bottom pins.
        // (Only primary names are shown in the single center column per top/bottom pin.)
        let top_name_rows = self.max_primary_name_len(top_pins);
        let bot_name_rows = self.max_primary_name_len(bottom_pins);

        // Total number of pin number digits (for vertical display above/below body).
        let total = self.total();
        let pin_num_w = total.to_string().len();

        // Horizontal name widths for left/right pins.
        let (left_name_w, left_names_w) = self.max_name_widths_for(left_pins, show_alt);
        let (right_name_w, right_names_w) = self.max_name_widths_for(right_pins, show_alt);

        // Body dimensions derived from actual (possibly rotated) pin counts.
        let body_inner_h = left_pins.len();
        let body_inner_w = top_pins.len() * tb_col_w + 1; // 1 for extra right space
        let body_h = body_inner_h + 2; // 2 for top and bottom edge
        let body_w = body_inner_w + 1;

        // Top/bottom area heights.
        let top_pin_rows = if has_pins { pin_num_w } else { 0 };
        let bot_pin_rows = if has_pins { pin_num_w } else { 0 };
        let bqfp = self.style == QuadStyle::Bqfp;
        // BQFP 2-line bumpers need at least 2 rows above and below body.
        let top_area_h = (top_name_rows + pin_gap + top_pin_rows).max(if bqfp { 2 } else { 0 });
        let bot_area_h = (bot_pin_rows + pin_gap + bot_name_rows).max(if bqfp { 2 } else { 0 });

        // Left/right column widths.
        // +1 on lpin_w when no pins: one-space gap between name and left body border.
        // +1 on rpin_w when no pins: one-space gap between right body border and name.
        let lpin_w = if has_pins {
            left_name_w + pin_sep.len() + pin_num_w
        } else {
            left_name_w
        };
        let rpin_w = if has_pins {
            1 + pin_num_w + pin_sep.len() + right_name_w
        } else {
            1 + right_name_w
        };

        // Title row below all content.
        let title_rows = if has_pins { 1 } else { 0 };

        let canvas_h = top_area_h + body_h + bot_area_h + title_rows;
        let canvas_w = lpin_w + body_w + 2 + rpin_w;
        let body_col = lpin_w + 1;
        let body_row = top_area_h;

        let mut canvas = Canvas::new(canvas_w, canvas_h);

        // -- Body borders ---------------------------------------------------
        // For PLCC: notch corner and double-dash/pipe (-, |) follow direction
        let (tl, tr, bl, br, td, bd, rp, lp) = if self.style == QuadStyle::Plcc {
            if dir == Direction::North {
                ("/", "+", "+", "+", "-", " ", " ", " ")
            } else if dir == Direction::East {
                ("+", "\\", "+", "+", " ", " ", "|", " ")
            } else if dir == Direction::South {
                ("+", "+", "+", "/", " ", "-", " ", " ")
            } else { // Direction::West
                // pin 1 on left
                ("+", "+", "\\", "+", " ", " ", " ", "|")
            }
        } else {
            ("+", "+", "+", "+", " ", " ", " ", " ")
        };
        // For --bottom (L-R mirror), swap TL↔TR and BL↔BR corners and flip / ↔ \.
        let (tl, tr, bl, br, rp, lp) = if mirrored {
            (mirror_slash(tr), mirror_slash(tl), mirror_slash(br), mirror_slash(bl), lp, rp)
        } else {
            (tl, tr, bl, br, rp, lp)
        };
        for c in 1..body_w {
            canvas.set(body_col + c, body_row,              "-");
            canvas.set(body_col + c, body_row + 1,          td);
            canvas.set(body_col + c, body_row + body_h - 2, bd);
            canvas.set(body_col + c, body_row + body_h - 1, "-");
        }
        for r in 1..body_h - 1 {
            canvas.set(body_col,              body_row + r, "|");
            canvas.set(body_col + 1,          body_row + r, lp);
            canvas.set(body_col + body_w - 1, body_row + r, rp);
            canvas.set(body_col + body_w,     body_row + r, "|");
        }
        if td == "-" {
            canvas.set(body_col + 1,          body_row + 1, td);
            canvas.set(body_col + body_w - 1, body_row + 1, td);
        }
        if bd == "-" {
            canvas.set(body_col + 1,          body_row + body_h - 2, bd);
            canvas.set(body_col + body_w - 1, body_row + body_h - 2, bd);
        }
        if tl == "+" {
            canvas.set(body_col,     body_row, tl);
        } else {
            canvas.set(body_col + 1, body_row, tl);
        }
        if tr == "+" {
            canvas.set(body_col + body_w,     body_row, tr);
        } else {
            canvas.set(body_col + body_w - 1, body_row, tr);
        }
        if bl == "+" {
            canvas.set(body_col,     body_row + body_h - 1, bl);
        } else {
            canvas.set(body_col + 1, body_row + body_h - 1, bl);
        }
        if br == "+" {
            canvas.set(body_col + body_w,     body_row + body_h - 1, br);
        } else {
            canvas.set(body_col + body_w - 1, body_row + body_h - 1, br);
        }

        // BQFP bumper corners: two diagonal lines at each corner.
        // Characters are always \ at TL/BR and / at TR/BL — the body position does not move
        // between top and bottom views, so the physical corner shapes are the same.
        if self.style == QuadStyle::Bqfp {
            let (tl_b, tr_b, bl_b, br_b) = ("\\", "/", "/", "\\");
            canvas.set(body_col - 1,          body_row - 1,          tl_b);
            canvas.set(body_col + body_w + 1, body_row - 1,          tr_b);
            canvas.set(body_col - 1,          body_row + body_h,     bl_b);
            canvas.set(body_col + body_w + 1, body_row + body_h,     br_b);
            canvas.set(body_col - 2,          body_row - 2,          tl_b);
            canvas.set(body_col + body_w + 2, body_row - 2,          tr_b);
            canvas.set(body_col - 2,          body_row + body_h + 1, bl_b);
            canvas.set(body_col + body_w + 2, body_row + body_h + 1, br_b);
        }

        // -- IC name/title centered in body interior (use title if it fits, else name) --
        {
            let display_name = if self.title.graphemes(true).count() <= body_inner_w {
                &self.title
            } else {
                &self.name
            };
            let display_len = display_name.graphemes(true).count();
            if display_len > 0 {
                let name_col = body_col + 1 + body_inner_w.saturating_sub(display_len) / 2;
                let name_row = body_row + 1 + body_inner_h / 2;
                canvas.draw_text(name_col, name_row, display_name);
            }
        }

        // -- Pin 1 marker: find pin 1 in the layout, place * at nearest interior cell --
        let (star_col, star_row) = find_pin1_star(
            top_pins, right_pins, bottom_pins, left_pins,
            body_col, body_row, body_h, body_w, tb_col_w,
        );
        canvas.set(star_col, star_row, "*");

        // -- Top pin names (vertical, bottom-justified) and numbers --------
        for (i, &p) in top_pins.iter().enumerate() {
            let center_col = body_col + 1 + i * tb_col_w + tb_col_w / 2;
            // Primary name, bottom-justified in top_name_rows rows
            let name = self.pin(p).name().to_string();
            let gs: Vec<String> = name.graphemes(true).map(String::from).collect();
            let start_row = top_name_rows.saturating_sub(gs.len());
            for (j, g) in gs.iter().enumerate() {
                canvas.set(center_col, start_row + j, g);
            }
            if has_pins {
                let pin_str = p.to_string();
                let pgs: Vec<String> = pin_str.graphemes(true).map(String::from).collect();
                let start_row =
                    top_name_rows + pin_gap + pin_num_w.saturating_sub(pgs.len());
                for (j, g) in pgs.iter().enumerate() {
                    canvas.set(center_col, start_row + j, g);
                }
            }
        }

        // -- Bottom pin names (vertical, top-justified) and numbers --------
        let bot_area_start = body_row + body_h;
        let bot_name_start = bot_area_start + if has_pins { bot_pin_rows + pin_gap } else { 0 };
        for (i, &p) in bottom_pins.iter().enumerate() {
            let center_col = body_col + 1 + i * tb_col_w + tb_col_w / 2;
            if has_pins {
                let pin_str = p.to_string();
                let pgs: Vec<String> = pin_str.graphemes(true).map(String::from).collect();
                for (j, g) in pgs.iter().enumerate() {
                    canvas.set(center_col, bot_area_start + j, g);
                }
            }
            let name = self.pin(p).name().to_string();
            let gs: Vec<String> = name.graphemes(true).map(String::from).collect();
            for (j, g) in gs.iter().enumerate() {
                canvas.set(center_col, bot_name_start + j, g);
            }
        }

        // -- Left pin names and optional pin numbers (right-justified) -----
        for (i, &p) in left_pins.iter().enumerate() {
            let row = body_row + 1 + i;
            let name_str = self.pin(p).names_horizontal(&left_names_w, true);
            let text = if has_pins {
                format!("{}{}{:>width$}", name_str, pin_sep, p, width = pin_num_w)
            } else {
                name_str
            };
            let text_len = text.graphemes(true).count();
            let start_col = lpin_w.saturating_sub(text_len);
            canvas.draw_text(start_col, row, &text);
        }

        // -- Right pin names and optional pin numbers (left-justified) -----
        // Start 1 column past the border to leave a gap between | and the text.
        for (i, &p) in right_pins.iter().enumerate() {
            let row = body_row + 1 + i;
            let name_str = self.pin(p).names_horizontal(&right_names_w, false);
            let text = if has_pins {
                format!("{:<width$}{}{}", p, pin_sep, name_str, width = pin_num_w)
            } else {
                name_str
            };
            canvas.draw_text(body_col + body_w + 2, row, &text);
        }

        // -- Title centered below all content ------------------------------
        if has_pins {
            let title_row = bot_area_start + bot_area_h;
            let title_len = self.title.graphemes(true).count();
            let title_col = canvas_w.saturating_sub(title_len) / 2;
            canvas.draw_text(title_col, title_row, &self.title);
        }

        canvas
    }

    /// Max grapheme count of the primary name across the given pins.
    fn max_primary_name_len(&self, pins: &[usize]) -> usize {
        pins.iter()
            .map(|&p| self.pin(p).name().graphemes(true).count())
            .max()
            .unwrap_or(0)
    }

    /// Per-index max width for horizontal name display, respecting show_alt.
    /// Returns (total_width, per_index_widths).
    fn max_name_widths_for(
        &self,
        pins: &[usize],
        show_alt: AltNames,
    ) -> (usize, Vec<usize>) {
        let limit = match show_alt {
            AltNames::None => 1,
            AltNames::Alt1 => 2,
            AltNames::Alt2 => 3,
            AltNames::All => usize::MAX,
        };
        let mut names_width: Vec<usize> = Vec::new();
        for &pin in pins {
            for (i, name) in self.pin(pin).names().iter().enumerate() {
                if i >= limit {
                    break;
                }
                let width = name.graphemes(true).count();
                if names_width.len() <= i {
                    names_width.push(0);
                }
                if names_width[i] < width {
                    names_width[i] = width;
                }
            }
        }
        if names_width.is_empty() {
            return (0, vec![]);
        }
        let sum: usize = names_width.iter().sum();
        let spaces = names_width.len() - 1;
        (sum + spaces, names_width)
    }
}

/// Reversed clone of a slice.
fn rev(v: &[usize]) -> Vec<usize> {
    v.iter().rev().cloned().collect()
}

/// Flip ASCII diagonal: `/` ↔ `\`, other chars unchanged.
fn mirror_slash(s: &'static str) -> &'static str {
    match s {
        "/" => "\\",
        "\\" => "/",
        _ => s,
    }
}

/// Find the body interior cell nearest to pin 1 given the 4-edge pin layout.
fn find_pin1_star(
    top_pins: &[usize],
    right_pins: &[usize],
    bottom_pins: &[usize],
    left_pins: &[usize],
    body_col: usize,
    body_row: usize,
    body_h: usize,
    body_w: usize,
    tb_col_w: usize,
) -> (usize, usize) {
    if let Some(i) = top_pins.iter().position(|&p| p == 1) {
        // Pin 1 on top edge: place star at top interior row, pin's center column
        let col = body_col + 1 + i * tb_col_w + tb_col_w / 2;
        (col, body_row + 1)
    } else if let Some(i) = right_pins.iter().position(|&p| p == 1) {
        // Pin 1 on right edge: rightmost interior column, pin's row
        (body_col + body_w - 1, body_row + 1 + i)
    } else if let Some(i) = bottom_pins.iter().position(|&p| p == 1) {
        // Pin 1 on bottom edge: bottom interior row, pin's center column
        let col = body_col + 1 + i * tb_col_w + tb_col_w / 2;
        (col, body_row + body_h - 2)
    } else {
        // Pin 1 on left edge: leftmost interior column, pin's row
        let i = left_pins.iter().position(|&p| p == 1).unwrap_or(0);
        (body_col + 1, body_row + 1 + i)
    }
}

impl fmt::Display for Quad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[name={} ", self.name)?;
        write!(f, "title={} ", self.title)?;
        write!(
            f,
            "package={:?}{} ",
            self.style,
            2 * self.top + 2 * self.side
        )?;
        write!(f, "pins=[")?;
        let total = self.total();
        for pin in 1..total {
            write!(f, "{} ", self.pin(pin).name())?;
        }
        write!(f, "{}]]", self.pin(total).name())
    }
}

impl FromStr for Quad {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match toml::from_str::<Value>(s.trim()) {
            Err(err) => return Err(err.to_string()),
            Ok(p) => p,
        };
        let toml = v.as_table().unwrap();

        let name = match toml.get("name") {
            None => return Err("no name".to_string()),
            Some(v) => match v.as_str() {
                None => return Err("name must be string".to_string()),
                Some(str) => str.to_string(),
            },
        };

        let title = match toml.get("title") {
            None => name.to_string(),
            Some(v) => match v.as_str() {
                None => return Err("title must be string".to_string()),
                Some(str) => str.to_string(),
            },
        };

        let (style, top, side) = parse_package_type(toml)?;

        let pin1 = match toml.get("pin1") {
            None => Pin1Pos::TopCenter,
            Some(v) => match v.as_str() {
                None => return Err("pin1 must be string".to_string()),
                Some("top-center") => Pin1Pos::TopCenter,
                Some("left-top") => Pin1Pos::LeftTop,
                Some("bottom-left") => Pin1Pos::BottomLeft,
                Some(s) => return Err(format!("unknown pin1 position {:?}", s)),
            },
        };

        let total = 2 * top + 2 * side;
        match pins_from_toml(toml, total) {
            Err(err) => Err(err),
            Ok(pins) => Ok(Quad { name, title, style, top, side, pin1, pins }),
        }
    }
}

fn parse_package_type(toml: &Map<String, Value>) -> Result<(QuadStyle, usize, usize), String> {
    let plcc = toml.get("plcc");
    let qfp = toml.get("qfp");
    let bqfp = toml.get("bqfp");

    match (plcc, qfp, bqfp) {
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) | (_, Some(_), Some(_)) => {
            Err("cannot specify more than one of plcc, qfp, bqfp".to_string())
        }
        (None, None, None) => Err("no plcc, qfp, or bqfp package".to_string()),
        (Some(v), None, None) => parse_plcc(v),
        (None, Some(v), None) => parse_qfp(v, QuadStyle::Qfp),
        (None, None, Some(v)) => parse_qfp(v, QuadStyle::Bqfp),
    }
}

fn parse_plcc(v: &Value) -> Result<(QuadStyle, usize, usize), String> {
    let n = v.as_integer().ok_or_else(|| "plcc must be a number".to_string())?;
    if n <= 0 || n % 4 != 0 {
        return Err(format!("plcc {} must be a positive multiple of 4", n));
    }
    let n = n as usize;
    Ok((QuadStyle::Plcc, n / 4, n / 4))
}

fn parse_qfp(v: &Value, style: QuadStyle) -> Result<(QuadStyle, usize, usize), String> {
    let pkg = match style {
        QuadStyle::Bqfp => "bqfp",
        _ => "qfp",
    };
    match v {
        Value::Integer(n) => {
            let n = *n;
            if n <= 0 || n % 4 != 0 {
                return Err(format!("{} {} must be a positive multiple of 4", pkg, n));
            }
            let n = n as usize;
            Ok((style, n / 4, n / 4))
        }
        Value::String(s) => {
            let parts: Vec<&str> = s.splitn(2, 'x').collect();
            if parts.len() != 2 {
                return Err(format!("{} {:?} must be NxM", pkg, s));
            }
            let top: usize = parts[0]
                .parse()
                .map_err(|_| format!("{} {:?}: N must be a number", pkg, s))?;
            let side: usize = parts[1]
                .parse()
                .map_err(|_| format!("{} {:?}: M must be a number", pkg, s))?;
            if top == 0 || side == 0 {
                return Err(format!("{} {:?}: N and M must be positive", pkg, s));
            }
            Ok((style, top, side))
        }
        _ => Err(format!("{} must be a number or NxM string", pkg)),
    }
}

fn pins_from_toml(
    toml: &Map<String, Value>,
    total: usize,
) -> Result<BTreeMap<usize, PinName>, String> {
    let mut pins = BTreeMap::new();
    for key in toml.keys() {
        if let Ok(n) = key.parse::<usize>() {
            if n == 0 {
                return Err("invalid pin number 0".to_string());
            }
            if n > total {
                return Err(format!(
                    "pin number {} must not be greater than total {}",
                    n, total
                ));
            }
            match toml.get(key).unwrap().as_str() {
                None => return Err(format!("name for pin {} must be string", n)),
                Some(name) => pins.insert(n, PinName::from_str(name).unwrap()),
            };
        }
    }
    for p in 1..=total {
        if !pins.contains_key(&p) {
            return Err(format!("missing pin {} definition", p));
        }
    }
    Ok(pins)
}
