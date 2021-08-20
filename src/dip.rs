extern crate unicode_segmentation;

use crate::cli::{Direction, Side};
use core::iter::Iterator;
use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use toml::map::Map;
use toml::Value;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DipWidth {
    MIL300,
    MIL600,
}

#[derive(Debug, PartialEq)]
pub struct Dip {
    pub name: String,            // IC name
    pub name_chars: Vec<String>, // Grapheme list of name
    pub title: String,           // IC title
    pub dip: usize,              // pin count
    pub width: DipWidth,         // package width
    pub pins: Vec<String>,       // names of pins
}

impl Dip {
    pub fn print(&self, dir: Direction, side: Side, show_pin: bool) -> Vec<String> {
        return match dir {
            Direction::NORTH => {
                self.print_vertical(side, show_pin, 1, self.dip / 2, self.dip, self.dip / 2 + 1)
            }
            Direction::SOUTH => {
                self.print_vertical(side, show_pin, self.dip / 2 + 1, self.dip, self.dip / 2, 1)
            }
            Direction::EAST => {
                self.print_horizontal(side, show_pin, self.dip / 2, 1, self.dip / 2 + 1, self.dip)
            }
            Direction::WEST => {
                self.print_horizontal(side, show_pin, self.dip, self.dip / 2 + 1, 1, self.dip / 2)
            }
        };
    }

    fn print_vertical(
        &self,
        side: Side,
        show_pin: bool,
        left_start: usize,
        left_end: usize,
        right_start: usize,
        right_end: usize,
    ) -> Vec<String> {
        let (lstart, lend, rstart, rend) = match side {
            Side::TOP => (left_start, left_end, right_start, right_end),
            Side::BOTTOM => (right_start, right_end, left_start, left_end),
        };
        let lmax = self.max_name_len(lstart, lend);
        let rmax = self.max_name_len(rstart, rend);
        let lpin = if show_pin {
            self.max_pin_len(lstart, lend) + 2
        } else {
            0
        };
        let rpin = if show_pin {
            self.max_pin_len(rstart, rend) + 2
        } else {
            0
        };
        let width = lmax + lpin + 1 + self.dip_width() + 1 + rpin + rmax;

        let mut out = Vec::new();
        let mut line = String::new();
        line.push_str(&print_right(lmax + lpin + 1, " "));
        line.push_str(&print_chars(self.dip_width(), '_'));
        out.push(line);

        let num_chars = self.name_chars.len();
        let top = (self.dip / 2 - num_chars) / 2 + 1;
        let bottom = top + num_chars;
        let center = self.dip_width() / 2;
        let right = self.dip_width() - 1;

        let mut l = lstart;
        let mut r = rstart;
        for pos in 1..=self.dip / 2 {
            let mut line = String::new();
            line.push_str(&print_right(lmax, &self.pins[l]));
            if show_pin {
                line.push_str(&format!("{pin:>width$}", pin = l, width = lpin));
            }

            let spc = if pos == self.dip / 2 { '_' } else { ' ' };
            line.push('|');
            for c in 0..self.dip_width() {
                if l == 1 && c == 0 {
                    line.push('*');
                } else if r == 1 && c == right {
                    line.push('*');
                } else if c == center && (pos >= top && pos < bottom) && side == Side::TOP {
                    line += &self.name_chars[pos - top];
                } else {
                    line.push(spc);
                }
            }
            line.push('|');

            if show_pin {
                line.push_str(&format!("{pin:<width$}", pin = r, width = rpin));
            }
            line.push_str(&print_left(rmax, &self.pins[r]));
            out.push(line);

            l = pin_step(l, lstart, lend);
            r = pin_step(r, rstart, rend);
        }
        if show_pin {
            out.push(print_center(width, &self.title));
        }
        return out;
    }

    fn dip_width(&self) -> usize {
        return match self.width {
            DipWidth::MIL300 => 5,
            DipWidth::MIL600 => 9,
        };
    }

    fn print_horizontal(
        &self,
        side: Side,
        show_pin: bool,
        top_start: usize,
        top_end: usize,
        bottom_start: usize,
        bottom_end: usize,
    ) -> Vec<String> {
        let (tstart, tend, bstart, bend) = match side {
            Side::TOP => (top_start, top_end, bottom_start, bottom_end),
            Side::BOTTOM => (bottom_start, bottom_end, top_start, top_end),
        };
        let height = self.dip_height();

        let mut out = Vec::new();
        self.print_pins(tstart, tend, show_pin, true, &mut out);
        let center = height / 2 + 1;
        let left = (self.dip - self.name_chars.len()) / 2;
        let right = left + self.name_chars.len();
        for l in 1..=height {
            let mut line = String::new();
            let print_name = side == Side::TOP && l == center;
            let mut t = tstart;
            let mut b = bstart;
            for pos in 1..=self.dip / 2 {
                let c = (pos - 1) * 2;
                line.push_str(match pos {
                    1 if l == 1 || l == height => "+",
                    1 => "|",
                    _ if l == 1 || l == height => "-",
                    _ if print_name && c >= left && c < right => &self.name_chars[c - left],
                    _ => " ",
                });
                line.push_str(match pos {
                    1 if l == 1 || l == height => "-",
                    1 if l == 2 && t == 1 => "*",
                    1 if l == height - 1 && b == 1 => "*",
                    _ if l == 1 || l == height => "-",
                    _ if l == 2 && t == 1 => "*",
                    _ if l == height - 1 && b == 1 => "*",
                    _ if print_name && c + 1 >= left && c + 1 < right => {
                        &self.name_chars[c + 1 - left]
                    }
                    _ => " ",
                });
                t = pin_step(t, tstart, tend);
                b = pin_step(b, bstart, bend);
            }
            line.push_str(match l {
                1 => "+",
                _ if l == height => "+",
                _ => "|",
            });
            out.push(line);
        }
        self.print_pins(bstart, bend, show_pin, false, &mut out);

        if show_pin {
            out.push(print_center(self.dip + 1, &self.title));
        }
        return out;
    }

    fn print_pins(
        &self,
        start: usize,
        end: usize,
        show_pin: bool,
        top: bool,
        out: &mut Vec<String>,
    ) {
        let name_max = self.max_name_len(start, end);
        let pin_width = if show_pin {
            self.max_pin_len(start, end)
        } else {
            0
        };
        if top {
            let mut lines = vec![String::new(); name_max];
            let mut pin = start;
            for _ in 1..=self.dip / 2 {
                let pin_chars = &self.pins[pin]
                    .graphemes(true)
                    .map(String::from)
                    .collect::<Vec<String>>();
                for l in 0..name_max {
                    let line = &mut lines[l];
                    line.push(' ');
                    if l < pin_chars.len() {
                        line.push_str(&pin_chars[pin_chars.len() - l - 1]);
                    } else {
                        line.push(' ')
                    }
                }
                pin = pin_step(pin, start, end);
            }
            for l in (0..name_max).rev() {
                out.push(String::from(&lines[l]));
            }
            if show_pin {
                out.push(String::from(""));
                for l in 0..pin_width {
                    let mut line = String::new();
                    let mut pin = start;
                    for _ in 1..=self.dip / 2 {
                        line.push(' ');
                        line.push(match l {
                            0 if pin < 10 => ' ',
                            0 => char::from_digit((pin / 10) as u32, 10).unwrap(),
                            _ => char::from_digit((pin % 10) as u32, 10).unwrap(),
                        });
                        pin = pin_step(pin, start, end);
                    }
                    out.push(line);
                }
            }
        } else {
            if show_pin {
                for l in 0..pin_width {
                    let mut line = String::new();
                    let mut pin = start;
                    for _ in 1..=self.dip / 2 {
                        line.push(' ');
                        line.push(match l {
                            0 if pin < 10 => char::from_digit(pin as u32, 10).unwrap(),
                            0 => char::from_digit((pin / 10) as u32, 10).unwrap(),
                            1 if pin < 10 => ' ',
                            _ => char::from_digit((pin % 10) as u32, 10).unwrap(),
                        });
                        pin = pin_step(pin, start, end);
                    }
                    out.push(line);
                }
                out.push(String::from(""));
            }
            let mut lines = vec![String::new(); name_max];
            let mut pin = start;
            for _ in 1..=self.dip / 2 {
                let pin_chars = &self.pins[pin]
                    .graphemes(true)
                    .map(String::from)
                    .collect::<Vec<String>>();
                for l in 0..name_max {
                    let line = &mut lines[l];
                    line.push(' ');
                    if l < pin_chars.len() {
                        line.push_str(&pin_chars[l]);
                    } else {
                        line.push(' ')
                    }
                }
                pin = pin_step(pin, start, end);
            }
            for l in 0..name_max {
                out.push(String::from(&lines[l]));
            }
        }
    }

    fn dip_height(&self) -> usize {
        return match self.width {
            DipWidth::MIL300 => 4,
            DipWidth::MIL600 => 6,
        };
    }

    fn max_name_len(&self, start: usize, end: usize) -> usize {
        let mut name_width = 0;
        for p in min(start, end)..=max(start, end) {
            let len = self.pins[p].graphemes(true).count();
            if len > name_width {
                name_width = len;
            }
        }
        return name_width;
    }

    fn max_pin_len(&self, start: usize, end: usize) -> usize {
        return if start < 10 && end < 10 { 1 } else { 2 };
    }
}

fn pin_step(pin: usize, start: usize, end: usize) -> usize {
    return if start < end { pin + 1 } else { pin - 1 };
}

fn print_center(width: usize, text: &str) -> String {
    let mut out = String::new();
    if width >= text.len() {
        out.push_str(&print_spaces((width - text.len()) / 2));
    }
    out.push_str(text);
    return out;
}

fn print_left(width: usize, text: &str) -> String {
    let mut out = String::from(text);
    if width >= text.len() {
        out.push_str(&print_spaces(width - text.len()));
    }
    return out;
}

fn print_right(width: usize, text: &str) -> String {
    let mut out = String::new();
    if width >= text.len() {
        out.push_str(&print_spaces(width - text.len()));
    }
    out.push_str(text);
    return out;
}

fn print_spaces(width: usize) -> String {
    return print_chars(width, ' ');
}

fn print_chars(width: usize, c: char) -> String {
    let mut out = String::new();
    for _ in 0..width {
        out.push(c);
    }
    return out;
}

impl fmt::Display for Dip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[name={} ", self.name)?;
        write!(f, "title={} ", self.title)?;
        write!(f, "package=DIP{} withd={:?} ", self.dip, self.width)?;
        write!(f, "pins=[")?;
        for pin in 1..self.dip {
            write!(f, "{} ", self.pins[pin])?;
        }
        write!(f, "{}]]", self.pins[self.dip])
    }
}

impl FromStr for Dip {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s.parse::<Value>() {
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
        let name_chars = name
            .graphemes(true)
            .map(String::from)
            .collect::<Vec<String>>();

        let title = match toml.get("title") {
            None => name.to_string(),
            Some(v) => match v.as_str() {
                None => return Err("title must be string".to_string()),
                Some(str) => str.to_string(),
            },
        };

        let dip = match toml.get("dip") {
            None => return Err("no dip package".to_string()),
            Some(v) => match v.as_integer() {
                None => return Err("dip package must be number".to_string()),
                Some(n) if n <= 0 => return Err(format!("dip package {} must be positive", n)),
                Some(n) if n >= 50 => {
                    return Err(format!("dip package {} must be less than 50", n))
                }
                Some(n) if n % 2 != 0 => return Err(format!("dip package {} must be even", n)),
                Some(n) => usize::try_from(n).ok().unwrap(),
            },
        };

        let width = match toml.get("width") {
            None => return Err("no width".to_string()),
            Some(v) => match v.as_integer() {
                None => return Err("width must be number in mil".to_string()),
                Some(w) if w == 300 => DipWidth::MIL300,
                Some(w) if w == 600 => DipWidth::MIL600,
                Some(w) => return Err(format!("width {} must be 300 or 600 mil", w)),
            },
        };

        match Dip::pins_to_vec_result(toml, dip) {
            Err(err) => return Err(err),
            Ok(pins) => {
                return Ok(Dip {
                    name,
                    name_chars,
                    title,
                    dip,
                    width,
                    pins,
                })
            }
        };
    }
}

impl Dip {
    fn pins_to_vec_result(toml: &Map<String, Value>, dip: usize) -> Result<Vec<String>, String> {
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
                    Some(name) => pins.insert(n, name.to_string()),
                };
            }
        }

        let mut names = Vec::with_capacity(dip + 1);
        names.push("<pin0>".to_string());
        for p in 1..=dip {
            if !pins.contains_key(&p) {
                return Err(format!("missing pin {} definition", p));
            }
            names.push(pins.get(&p).unwrap().to_string());
        }
        return Ok(names);
    }
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
    assert_eq!(DipWidth::MIL300, dip.width);
    assert_eq!("VDD", dip.pins[1]);
    assert_eq!("PA6", dip.pins[2]);
    assert_eq!("PA7", dip.pins[3]);
    assert_eq!("PA1", dip.pins[4]);

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
fn test_decode_error() {
    assert_eq!(Dip::from_str("# empty").err(), Some("no name".to_string()));
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
        Dip::from_str("name = \"SN7400\"\ndip = 50").err(),
        Some("dip package 50 must be less than 50".to_string())
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
        Dip::from_str("name = \"SN7400\"\ndip = 8\nwidth = 400").err(),
        Some("width 400 must be 300 or 600 mil".to_string())
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
    assert_eq!(
        Dip::from_str(
            r#"
            name = "SN7400"
            dip = 8
            width = 300
            1 = "pin1"
            1 = "pin2"
         "#
        )
        .err(),
        Some("duplicate key: `1` at line 1 column 1".to_string())
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
