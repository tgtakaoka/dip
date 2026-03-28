use crate::canvas::Canvas;
use crate::cli::{AltNames, Direction, PinGap, Side};
use crate::pin::PinName;
use crate::print;
use std::cmp::max;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use toml::map::Map;
use toml::Value;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
pub struct Dip {
    pub name: String,
    pub title: String,
    pub dip: usize,
    pub width: DipWidth,
    pins: BTreeMap<usize, PinName>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DipWidth {
    Mil300,
    Mil500,
    Mil600,
    Mil750,
    Mil900,
    Mil1300,
}

impl Dip {
    pub fn print(
        &self,
        dir: Direction,
        side: Side,
        show_pin: PinGap,
        show_alt: AltNames,
    ) -> Vec<String> {
        // Step 1: canonical Top layout (always)
        let (top, bot) = self.north_pins();

        // Step 2: direction + side → select the two pin arrays for rendering.
        // Side::Bottom = left-right mirror of the same-direction Top output:
        //   horizontal edges (top/bot) are reversed; vertical edges (left/right) swap without reversing.
        let canvas = match (dir, side) {
            (Direction::North, Side::Top) =>
                self.render_horizontal(&top, &bot, show_pin, show_alt),
            (Direction::North, Side::Bottom) => {
                let t: Vec<usize> = top.iter().rev().cloned().collect();
                let b: Vec<usize> = bot.iter().rev().cloned().collect();
                self.render_horizontal(&t, &b, show_pin, show_alt)
            }
            (Direction::South, Side::Top) => {
                let t: Vec<usize> = bot.iter().rev().cloned().collect();
                let b: Vec<usize> = top.iter().rev().cloned().collect();
                self.render_horizontal(&t, &b, show_pin, show_alt)
            }
            (Direction::South, Side::Bottom) =>
                self.render_horizontal(&bot, &top, show_pin, show_alt),
            (Direction::East, Side::Top) =>
                self.render_vertical(&bot, &top, show_pin, show_alt),
            (Direction::East, Side::Bottom) =>
                self.render_vertical(&top, &bot, show_pin, show_alt),
            (Direction::West, Side::Top) => {
                let l: Vec<usize> = top.iter().rev().cloned().collect();
                let r: Vec<usize> = bot.iter().rev().cloned().collect();
                self.render_vertical(&l, &r, show_pin, show_alt)
            }
            (Direction::West, Side::Bottom) => {
                let l: Vec<usize> = bot.iter().rev().cloned().collect();
                let r: Vec<usize> = top.iter().rev().cloned().collect();
                self.render_vertical(&l, &r, show_pin, show_alt)
            }
        };
        canvas.render()
    }

    /// Returns (top_pins, bot_pins) in left→right order for the canonical North/Top view.
    fn north_pins(&self) -> (Vec<usize>, Vec<usize>) {
        let n = self.dip;
        (
            (n / 2 + 1..=n).rev().collect(), // n, n-1, …, n/2+1
            (1..=n / 2).collect(),             // 1, 2, …, n/2
        )
    }

    /// Draw a horizontal-body DIP (North or South orientation).
    /// top_pins: left→right across the top edge.
    /// bot_pins: left→right across the bottom edge.
    fn render_horizontal(
        &self,
        top_pins: &[usize],
        bot_pins: &[usize],
        show_pin: PinGap,
        show_alt: AltNames,
    ) -> Canvas {
        let (top_name_h, top_names_w) = self.max_name_width_for(top_pins, show_alt);
        let (bot_name_h, bot_names_w) = self.max_name_width_for(bot_pins, show_alt);
        let has_pins = show_pin != PinGap::None;
        let pin_gap: usize = 1;
        let top_pin_h = if has_pins { self.max_pin_width_for(top_pins) } else { 0 };
        let bot_pin_h = if has_pins { self.max_pin_width_for(bot_pins) } else { 0 };

        let body_h = self.dip_height();
        let canvas_w = self.dip + 1;

        let top_area_h = top_name_h + if has_pins { pin_gap + top_pin_h } else { 0 };
        let body_row = top_area_h;
        let bot_area_start = body_row + body_h;
        let bot_area_h = (if has_pins { bot_pin_h + pin_gap } else { 0 }) + bot_name_h;
        let title_h = if has_pins { 1 } else { 0 };
        let total_h = top_area_h + body_h + bot_area_h + title_h;

        let mut canvas = Canvas::new(canvas_w, total_h);

        // Top pin names (bottom-justified) and optional pin numbers
        for (p_idx, &pin) in top_pins.iter().enumerate() {
            let col = p_idx * 2 + 1;
            let name_chars = self.pin(pin).names_vertical(&top_names_w, true);
            for (r, ch) in name_chars.iter().enumerate() {
                canvas.set(col, r, ch);
            }
            if has_pins {
                let pin_chars = print::bottom(top_pin_h, &pin.to_string());
                for (r, ch) in pin_chars.iter().enumerate() {
                    canvas.set(col, top_name_h + pin_gap + r, ch);
                }
            }
        }

        // Body borders
        canvas.set(0, body_row, "+");
        canvas.hline(1, body_row, self.dip - 1, '-');
        canvas.set(self.dip, body_row, "+");
        canvas.vline(0, body_row + 1, body_h - 2, '|');
        canvas.vline(self.dip, body_row + 1, body_h - 2, '|');
        canvas.set(0, body_row + body_h - 1, "+");
        canvas.hline(1, body_row + body_h - 1, self.dip - 1, '-');
        canvas.set(self.dip, body_row + body_h - 1, "+");

        // IC name/title centered in body interior (use title if it fits, else name)
        let body_inner_w = self.dip - 1;
        let display_name = if self.title.graphemes(true).count() <= body_inner_w {
            &self.title
        } else {
            &self.name
        };
        let display_len = display_name.graphemes(true).count();
        if display_len > 0 {
            let name_col = 1 + body_inner_w.saturating_sub(display_len) / 2;
            let name_row = body_row + body_h / 2;
            canvas.draw_text(name_col, name_row, display_name);
        }

        // Pin 1 marker in body interior corner nearest pin 1
        let (star_col, star_row) = self.find_pin1_h(top_pins, bot_pins, body_row, body_h);
        canvas.set(star_col, star_row, "*");

        // Bottom pin numbers (closest to body) and pin names (below those)
        let bot_name_start_row = bot_area_start + if has_pins { bot_pin_h + pin_gap } else { 0 };
        for (p_idx, &pin) in bot_pins.iter().enumerate() {
            let col = p_idx * 2 + 1;
            if has_pins {
                let pin_chars = print::top(bot_pin_h, &pin.to_string());
                for (r, ch) in pin_chars.iter().enumerate() {
                    canvas.set(col, bot_area_start + r, ch);
                }
            }
            let name_chars = self.pin(pin).names_vertical(&bot_names_w, false);
            for (r, ch) in name_chars.iter().enumerate() {
                canvas.set(col, bot_name_start_row + r, ch);
            }
        }

        // Title centered below all content
        if has_pins {
            let title_row = bot_area_start + bot_area_h;
            let title_len = self.title.graphemes(true).count();
            let title_col = canvas_w.saturating_sub(title_len) / 2;
            canvas.draw_text(title_col, title_row, &self.title);
        }

        canvas
    }

    /// Draw a vertical-body DIP (East or West orientation).
    /// left_pins: top→bottom along the left edge.
    /// right_pins: top→bottom along the right edge.
    fn render_vertical(
        &self,
        left_pins: &[usize],
        right_pins: &[usize],
        show_pin: PinGap,
        show_alt: AltNames,
    ) -> Canvas {
        let has_pins = show_pin != PinGap::None;
        let pin_sep = "  "; // 2 spaces between horizontal name and pin number

        let (left_name_w, left_names_w) = self.max_name_width_for(left_pins, show_alt);
        let (right_name_w, right_names_w) = self.max_name_width_for(right_pins, show_alt);

        let pin_num_w = max(
            left_pins.iter().max().copied().unwrap_or(1),
            right_pins.iter().max().copied().unwrap_or(1),
        )
        .to_string()
        .len();

        // +1 when no pins: one-space gap between pin name and body border on each side.
        let lpin_w =
            if has_pins { left_name_w + pin_sep.len() + pin_num_w } else { left_name_w };
        let rpin_w =
            if has_pins { pin_num_w + pin_sep.len() + right_name_w } else { right_name_w + 1 };

        // Body: width fixed by package mil-width; height = number of pins per side
        let body_inner_w = self.dip_width();
        let body_w = body_inner_w + 2;
        let body_inner_h = left_pins.len();
        let body_h = body_inner_h + 2;

        let canvas_w = lpin_w + body_w + 1 + rpin_w;
        let canvas_h = body_h;
        let body_col = lpin_w + 1;
        let body_row = 0;

        let mut canvas = Canvas::new(canvas_w, canvas_h);

        // Body borders
        canvas.set(body_col, body_row, "+");
        canvas.hline(body_col + 1, body_row, body_inner_w, '-');
        canvas.set(body_col + body_w - 1, body_row, "+");
        canvas.vline(body_col, body_row + 1, body_inner_h, '|');
        canvas.vline(body_col + body_w - 1, body_row + 1, body_inner_h, '|');
        canvas.set(body_col, body_row + body_h - 1, "+");
        canvas.hline(body_col + 1, body_row + body_h - 1, body_inner_w, '-');
        canvas.set(body_col + body_w - 1, body_row + body_h - 1, "+");

        // IC name/title stacked vertically in body interior (use title if it fits, else name)
        {
            let display_name = if self.title.graphemes(true).count() <= body_inner_h {
                &self.title
            } else {
                &self.name
            };
            let gs: Vec<String> = display_name.graphemes(true).map(String::from).collect();
            let name_v_len = gs.len();
            let center_col = body_col + 1 + body_inner_w / 2;
            let name_row_start = body_row + 1 + body_inner_h.saturating_sub(name_v_len) / 2;
            for (j, g) in gs.iter().enumerate() {
                canvas.set(center_col, name_row_start + j, g);
            }
        }

        // Pin 1 marker
        let (star_col, star_row) =
            self.find_pin1_v(left_pins, right_pins, body_col, body_row, body_w);
        canvas.set(star_col, star_row, "*");

        // Left pins (right-justified text ending at the body left border)
        for (i, &p) in left_pins.iter().enumerate() {
            let row = body_row + 1 + i;
            let name_str = self.pin(p).names_horizontal(&left_names_w, true);
            let full = if has_pins {
                format!("{}{}{:>width$}", name_str, pin_sep, p, width = pin_num_w)
            } else {
                name_str
            };
            let full_len = full.graphemes(true).count();
            let start_col = lpin_w.saturating_sub(full_len);
            canvas.draw_text(start_col, row, &full);
        }

        // Right pins (left-justified text starting after the body right border)
        // With pins: start immediately at the border; the pin number acts as separator.
        // Without pins: skip 1 column for a gap between | and the name.
        for (i, &p) in right_pins.iter().enumerate() {
            let row = body_row + 1 + i;
            let name_str = self.pin(p).names_horizontal(&right_names_w, false);
            let text = if has_pins {
                format!("{:<width$}{}{}", p, pin_sep, name_str, width = pin_num_w)
            } else {
                name_str
            };
            let col = if has_pins { body_col + body_w } else { body_col + body_w + 1 };
            canvas.draw_text(col, row, &text);
        }

        canvas
    }

    /// Returns the body interior cell (col, row) nearest to pin 1 for a horizontal body.
    fn find_pin1_h(
        &self,
        top_pins: &[usize],
        bot_pins: &[usize],
        body_row: usize,
        body_h: usize,
    ) -> (usize, usize) {
        if let Some(i) = top_pins.iter().position(|&p| p == 1) {
            (i * 2 + 1, body_row + 1)
        } else {
            let i = bot_pins.iter().position(|&p| p == 1).unwrap_or(0);
            (i * 2 + 1, body_row + body_h - 2)
        }
    }

    /// Returns the body interior cell (col, row) nearest to pin 1 for a vertical body.
    fn find_pin1_v(
        &self,
        left_pins: &[usize],
        right_pins: &[usize],
        body_col: usize,
        body_row: usize,
        body_w: usize,
    ) -> (usize, usize) {
        if let Some(i) = left_pins.iter().position(|&p| p == 1) {
            (body_col + 1, body_row + 1 + i)
        } else {
            let i = right_pins.iter().position(|&p| p == 1).unwrap_or(0);
            (body_col + body_w - 2, body_row + 1 + i)
        }
    }

    fn dip_height(&self) -> usize {
        match self.width {
            DipWidth::Mil300 => 4,
            DipWidth::Mil500 => 5,
            DipWidth::Mil600 => 6,
            DipWidth::Mil750 => 10,  // SDIP
            DipWidth::Mil900 => 9,
            DipWidth::Mil1300 => 13,
        }
    }

    fn dip_width(&self) -> usize {
        match self.width {
            DipWidth::Mil300 => 6,
            DipWidth::Mil500 => 7,
            DipWidth::Mil600 => 10,
            DipWidth::Mil750 => 19, // SDIP
            DipWidth::Mil900 => 17,
            DipWidth::Mil1300 => 26,
        }
    }

    fn pin(&self, pin_number: usize) -> &PinName {
        self.pins.get(&pin_number).unwrap()
    }

    /// Max width per name index across the given pins, respecting show_alt.
    /// Returns (total_horizontal_width, per_index_widths).
    fn max_name_width_for(
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

    /// Max number of digits in any pin number in the given set.
    fn max_pin_width_for(&self, pins: &[usize]) -> usize {
        pins.iter().max().copied().unwrap_or(1).to_string().len()
    }
}

impl fmt::Display for Dip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[name={} ", self.name)?;
        write!(f, "title={} ", self.title)?;
        write!(f, "package=DIP{} width={:?} ", self.dip, self.width)?;
        write!(f, "pins=[")?;
        for pin in 1..self.dip {
            write!(f, "{} ", self.pin(pin).name())?;
        }
        write!(f, "{}]]", self.pin(self.dip).name())
    }
}

impl FromStr for Dip {
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

        // Accept either "dip" or "sdip" key; "sdip" defaults width to Mil750.
        let is_sdip = toml.contains_key("sdip");
        let pkg_key = if is_sdip { "sdip" } else { "dip" };

        let dip = match toml.get(pkg_key) {
            None => return Err("no dip package".to_string()),
            Some(v) => match v.as_integer() {
                None => return Err("dip package must be number".to_string()),
                Some(n) if n <= 0 => return Err(format!("dip package {} must be positive", n)),
                Some(n) if n >= 80 => {
                    return Err(format!("dip package {} must be less than 80", n))
                }
                Some(n) if n % 2 != 0 => return Err(format!("dip package {} must be even", n)),
                Some(n) => usize::try_from(n).unwrap(),
            },
        };

        let width = match toml.get("width") {
            None => {
                if is_sdip {
                    DipWidth::Mil750
                } else {
                    return Err("no width".to_string());
                }
            }
            Some(v) => match v.as_integer() {
                None => return Err("width must be number in mil".to_string()),
                Some(300) => DipWidth::Mil300,
                Some(500) => DipWidth::Mil500,
                Some(600) => DipWidth::Mil600,
                Some(750) => DipWidth::Mil750,
                Some(900) => DipWidth::Mil900,
                Some(1300) => DipWidth::Mil1300,
                Some(w) => return Err(format!("unknown DIP width {}", w)),
            },
        };

        match pins_to_vec_result(toml, dip) {
            Err(err) => Err(err),
            Ok(pins) => Ok(Dip { name, title, dip, width, pins }),
        }
    }
}

fn pins_to_vec_result(
    toml: &Map<String, Value>,
    dip: usize,
) -> Result<BTreeMap<usize, PinName>, String> {
    let mut pins = BTreeMap::new();
    for pin in toml.keys() {
        if let Ok(n) = pin.parse::<usize>() {
            if n == 0 {
                return Err("invalid pin number 0".to_string());
            }
            if n > dip {
                return Err(format!(
                    "pin number {} must not be greater than dip {}",
                    n, dip
                ));
            }
            match toml.get(pin).unwrap().as_str() {
                None => return Err(format!("name for pin {} must be string", n)),
                Some(name) => pins.insert(n, PinName::from_str(name).unwrap()),
            };
        }
    }

    for p in 1..=dip {
        if !pins.contains_key(&p) {
            return Err(format!("missing pin {} definition", p));
        }
    }

    Ok(pins)
}

#[test]
fn test_dip_decode() {
    let dip = Dip::from_str(
        r#"
        name = "ATtiny412"
        title = "ATtiny412-SS"
        dip = 4
        width = 300
        4 = "PA1"
        2 = "PA6"
        3 = "PA7"
        1 = "VDD""#,
    )
    .unwrap();
    assert_eq!("ATtiny412", dip.name);
    assert_eq!("ATtiny412-SS", dip.title);
    assert_eq!(4, dip.dip);
    assert_eq!(DipWidth::Mil300, dip.width);
    assert_eq!("VDD", dip.pin(1).name());
    assert_eq!("PA6", dip.pin(2).name());
    assert_eq!("PA7", dip.pin(3).name());
    assert_eq!("PA1", dip.pin(4).name());

    assert_eq!(
        "ATtiny412",
        Dip::from_str(
            r#"
        name = "ATtiny412"
        dip = 4
        width = 300
        4 = "PA1"
        2 = "PA6"
        3 = "PA7"
        1 = "VDD""#
        )
        .unwrap()
        .title
    );
}

#[test]
fn test_sdip_decode() {
    let dip = Dip::from_str(
        r#"
        name = "TEST"
        sdip = 4
        1 = "A"
        2 = "B"
        3 = "C"
        4 = "D""#,
    )
    .unwrap();
    assert_eq!(DipWidth::Mil750, dip.width);

    // Explicit width overrides default
    let dip2 = Dip::from_str(
        r#"
        name = "TEST"
        sdip = 4
        width = 900
        1 = "A"
        2 = "B"
        3 = "C"
        4 = "D""#,
    )
    .unwrap();
    assert_eq!(DipWidth::Mil900, dip2.width);
}

#[test]
fn test_decode_error() {
    assert_eq!(Dip::from_str("dip = 4\nwidth = 300\n").err(), Some("no name".to_string()));
    assert_eq!(
        Dip::from_str("name = 7400").err(),
        Some("name must be string".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"").err(),
        Some("no dip package".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ntitle = 7400").err(),
        Some("title must be string".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ndip = \"14\"").err(),
        Some("dip package must be number".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ndip = -2").err(),
        Some("dip package -2 must be positive".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ndip = 80").err(),
        Some("dip package 80 must be less than 80".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ndip = 7").err(),
        Some("dip package 7 must be even".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ndip = 8").err(),
        Some("no width".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ndip = 8\nwidth = \"300\"").err(),
        Some("width must be number in mil".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ndip = 8\nwidth = 200").err(),
        Some("unknown DIP width 200".to_string())
    );
    assert_eq!(
        Dip::from_str("name = \"SN7400\"\ndip = 8\nwidth = 350").err(),
        Some("unknown DIP width 350".to_string())
    );
    assert_eq!(
        Dip::from_str(
            r#"
            name = "SN7400"
            dip = 8
            width = 300
            0 = "pin0"
         "#
        )
        .err(),
        Some("invalid pin number 0".to_string())
    );
    assert_eq!(
        Dip::from_str(
            r#"
            name = "SN7400"
            dip = 8
            width = 300
            9 = "pin9"
         "#
        )
        .err(),
        Some("pin number 9 must not be greater than dip 8".to_string())
    );
    assert_eq!(
        Dip::from_str(
            r#"
            name = "SN7400"
            dip = 8
            width = 300
            1 = true
         "#
        )
        .err(),
        Some("name for pin 1 must be string".to_string())
    );
    assert!(
        Dip::from_str(
            r#"
            name = "SN7400"
            dip = 8
            width = 300
            1 = "pin1"
            1 = "pin2"
         "#
        )
        .is_err()
    );
    assert_eq!(
        Dip::from_str(
            r#"
            name = "SN7400"
            dip = 8
            width = 300
            1 = "pin1"
            2 = "pin2"
         "#
        )
        .err(),
        Some("missing pin 3 definition".to_string())
    );
}
