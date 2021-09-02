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
struct PinName {
    names: String,
}

#[derive(Debug, PartialEq)]
pub struct Dip {
    pub name: String,               // IC name
    pub title: String,              // IC title
    pub dip: usize,                 // pin count
    pub width: DipWidth,            // package width
    pins: BTreeMap<usize, PinName>, // names of pins
}

impl PinName {
    fn names(&self) -> Vec<&str> {
        return self.names.split(',').map(str::trim).collect();
    }

    fn name(&self) -> &str {
        return self.names().get(0).unwrap();
    }

    fn names_horizontal(&self, show_alt: bool, names_max: &Vec<usize>, left: bool) -> String {
        if !show_alt {
            return self.name().to_string();
        }
        let column = names_max.len();
        let names: Vec<&str> = self.names();
        let mut line = String::new();
        for i in 0..column {
            if left {
                let n = column - i - 1;
                if n >= names.len() {
                    line.push_str(&print_spaces(names_max[n]));
                } else {
                    line.push_str(&print_right(names_max[n], names[n]));
                }
            } else {
                if i < names.len() {
                    line.push_str(&print_left(names_max[i], names[i]));
                }
            }
            if i < column - 1 {
                line.push(' ');
            }
        }
        return line;
    }

    fn names_vertical(&self, show_alt: bool, names_max: &Vec<usize>, top: bool) -> Vec<String> {
        if !show_alt {
            let height = names_max[0];
            return if top {
                print_bottom(height, self.name())
            } else {
                print_top(height, self.name())
            };
        }
        let column = names_max.len();
        let names: Vec<&str> = self.names();
        let mut out: Vec<String> = Vec::new();
        for i in 0..column {
            if top {
                let n = column - 1 - i;
                let text = if n < names.len() { names[n] } else { "" };
                out.append(&mut print_bottom(names_max[n], text));
            } else {
                let text = if i < names.len() { names[i] } else { "" };
                out.append(&mut print_top(names_max[i], text));
            }
            if i < column - 1 {
                out.push(String::from(" "));
            }
        }
        return out;
    }
}

impl Dip {
    pub fn print(&self, dir: Direction, side: Side, show_pin: bool, show_alt: bool) -> Vec<String> {
        return match dir {
            Direction::NORTH => self.print_vertical(
                side,
                show_pin,
                show_alt,
                1,
                self.dip / 2,
                self.dip,
                self.dip / 2 + 1,
            ),
            Direction::SOUTH => self.print_vertical(
                side,
                show_pin,
                show_alt,
                self.dip / 2 + 1,
                self.dip,
                self.dip / 2,
                1,
            ),
            Direction::EAST => self.print_horizontal(
                side,
                show_pin,
                show_alt,
                self.dip / 2,
                1,
                self.dip / 2 + 1,
                self.dip,
            ),
            Direction::WEST => self.print_horizontal(
                side,
                show_pin,
                show_alt,
                self.dip,
                self.dip / 2 + 1,
                1,
                self.dip / 2,
            ),
        };
    }

    fn print_vertical(
        &self,
        side: Side,
        show_pin: bool,
        show_alt: bool,
        left_start: usize,
        left_end: usize,
        right_start: usize,
        right_end: usize,
    ) -> Vec<String> {
        let (lstart, lend, rstart, rend) = match side {
            Side::TOP => (left_start, left_end, right_start, right_end),
            Side::BOTTOM => (right_start, right_end, left_start, left_end),
        };
        let (lmax, lmaxes) = self.max_name_len(lstart, lend, show_alt);
        let (rmax, rmaxes) = self.max_name_len(rstart, rend, show_alt);
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

        let mut out = Vec::new();
        let mut line = String::new();
        line.push_str(&print_right(lmax + lpin + 1, " "));
        line.push_str(&print_chars(self.dip_width(), '_'));
        out.push(line);

        let name_chars = self.name_chars();
        let num_chars = name_chars.len();
        let top = (self.dip / 2 - num_chars) / 2 + 1;
        let bottom = top + num_chars;
        let center = self.dip_width() / 2;
        let right = self.dip_width() - 1;

        let mut l = lstart;
        let mut r = rstart;
        for pos in 1..=self.dip / 2 {
            let mut line = String::new();
            line.push_str(&print_right(
                lmax,
                &self.pin(l).names_horizontal(show_alt, &lmaxes, true),
            ));
            if show_pin {
                line.push_str(&print_right(lpin, &l.to_string()));
            }

            let spc = if pos == self.dip / 2 { '_' } else { ' ' };
            line.push('|');
            for c in 0..self.dip_width() {
                let print_name = side == Side::TOP && c == center;
                if l == 1 && c == 0 {
                    line.push('*');
                } else if r == 1 && c == right {
                    line.push('*');
                } else if print_name && pos >= top && pos < bottom {
                    line += &name_chars[pos - top];
                } else {
                    line.push(spc);
                }
            }
            line.push('|');

            if show_pin {
                line.push_str(&print_left(rpin, &r.to_string()));
            }
            line.push_str(&print_left(
                rmax,
                &self.pin(r).names_horizontal(show_alt, &rmaxes, false),
            ));
            out.push(line);

            l = pin_step(l, lstart, lend);
            r = pin_step(r, rstart, rend);
        }
        if show_pin {
            let width = lmax + lpin + 1 + (self.dip_width() + self.title.len()) / 2;
            out.push(print_right(width, &self.title));
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
        show_alt: bool,
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
        self.print_pins_vertical(tstart, tend, show_pin, show_alt, true, &mut out);
        let center = height / 2 + 1;
        let name_chars = self.name_chars();
        let left = (self.dip - name_chars.len()) / 2;
        let right = left + name_chars.len();
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
                    _ if print_name && c >= left && c < right => &name_chars[c - left],
                    _ => " ",
                });
                line.push_str(match pos {
                    1 if l == 1 || l == height => "-",
                    1 if l == 2 && t == 1 => "*",
                    1 if l == height - 1 && b == 1 => "*",
                    _ if l == 1 || l == height => "-",
                    _ if l == 2 && t == 1 => "*",
                    _ if l == height - 1 && b == 1 => "*",
                    _ if print_name && c + 1 >= left && c + 1 < right => &name_chars[c + 1 - left],
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
        self.print_pins_vertical(bstart, bend, show_pin, show_alt, false, &mut out);

        if show_pin {
            let width = (self.dip + 1 + self.title.len()) / 2;
            out.push(print_right(width, &self.title));
        }
        return out;
    }

    fn print_pins_vertical(
        &self,
        start: usize,
        end: usize,
        show_pin: bool,
        show_alt: bool,
        top: bool,
        out: &mut Vec<String>,
    ) {
        let (name_max, names_max) = self.max_name_len(start, end, show_alt);
        let pin_width = self.max_pin_len(start, end);
        let mut names = vec![String::new(); name_max];
        let mut pins = vec![String::new(); pin_width];
        let mut pin = start;
        if top {
            for _ in 1..=self.dip / 2 {
                let pin_chars = self.pin(pin).names_vertical(show_alt, &names_max, top);
                for l in 0..name_max {
                    let line = &mut names[l];
                    line.push(' ');
                    line.push_str(&pin_chars[l]);
                }
                if show_pin {
                    let pin_number = print_bottom(pin_width, &pin.to_string());
                    for l in 0..pin_width {
                        let line = &mut pins[l];
                        line.push(' ');
                        line.push_str(&pin_number[l]);
                    }
                }
                pin = pin_step(pin, start, end);
            }
            out.append(&mut names);
            if show_pin {
                out.push(String::from(" "));
                out.append(&mut pins);
            }
        } else {
            for _ in 1..=self.dip / 2 {
                if show_pin {
                    let pin_number = print_top(pin_width, &pin.to_string());
                    for l in 0..pin_width {
                        let line = &mut pins[l];
                        line.push(' ');
                        line.push_str(&pin_number[l]);
                    }
                }
                let pin_chars = self.pin(pin).names_vertical(show_alt, &names_max, top);
                for l in 0..name_max {
                    let line = &mut names[l];
                    line.push(' ');
                    line.push_str(&pin_chars[l]);
                }
                pin = pin_step(pin, start, end);
            }
            if show_pin {
                out.append(&mut pins);
                out.push(String::from(" "));
            }
            out.append(&mut names);
        }
    }

    fn dip_height(&self) -> usize {
        return match self.width {
            DipWidth::MIL300 => 4,
            DipWidth::MIL600 => 6,
        };
    }

    fn name_chars(&self) -> Vec<String> {
        return self
            .name
            .graphemes(true)
            .map(String::from)
            .collect::<Vec<String>>();
    }

    fn pin(&self, pin_number: usize) -> &PinName {
        return &self.pins.get(&pin_number).unwrap();
    }

    fn max_name_len(&self, start: usize, end: usize, show_alt: bool) -> (usize, Vec<usize>) {
        let mut names_max = Vec::new();
        for p in min(start, end)..=max(start, end) {
            for (i, name) in self.pin(p).names().iter().enumerate() {
                let len = name.graphemes(true).count();
                if names_max.len() <= i {
                    names_max.push(0);
                }
                if names_max[i] < len {
                    names_max[i] = len;
                }
            }
        }
        return (
            if show_alt {
                let sum: usize = names_max.iter().sum();
                let spaces = names_max.len() - 1;
                sum + spaces
            } else {
                names_max[0]
            },
            names_max,
        );
    }

    fn max_pin_len(&self, start: usize, end: usize) -> usize {
        return if start < 10 && end < 10 { 1 } else { 2 };
    }
}

fn pin_step(pin: usize, start: usize, end: usize) -> usize {
    return if start < end { pin + 1 } else { pin - 1 };
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

fn print_top(height: usize, text: &str) -> Vec<String> {
    let mut out = text
        .graphemes(true)
        .map(String::from)
        .collect::<Vec<String>>();
    let len = out.len();
    for _ in 0..(height - len) {
        out.push(String::from(" "));
    }
    return out;
}

fn print_bottom(height: usize, text: &str) -> Vec<String> {
    let mut chars = text
        .graphemes(true)
        .map(String::from)
        .collect::<Vec<String>>();
    let len = chars.len();
    let mut out: Vec<String> = Vec::new();
    if height > len {
        for _ in 0..(height - len) {
            out.push(String::from(" "));
        }
    }
    out.append(&mut chars);
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
            write!(f, "{} ", self.pin(pin).name())?;
        }
        write!(f, "{}]]", self.pin(self.dip).name())
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

        match pins_to_vec_result(toml, dip) {
            Err(err) => return Err(err),
            Ok(pins) => {
                return Ok(Dip {
                    name,
                    title,
                    dip,
                    width,
                    pins,
                })
            }
        };
    }
}

impl FromStr for PinName {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return Ok(PinName {
            names: s.to_string(),
        });
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
    return Ok(pins);
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

#[test]
fn test_print_left_right() {
    assert_eq!(print_left(5, "AB"), "AB   ");
    assert_eq!(print_right(5, "AB"), "   AB");
}

#[test]
fn test_print_top_bottom() {
    assert_eq!(print_top(5, "AB"), vec!["A", "B", " ", " ", " "]);
    assert_eq!(print_bottom(5, "AB"), vec![" ", " ", " ", "A", "B"]);
}
