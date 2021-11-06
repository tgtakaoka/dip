extern crate unicode_segmentation;

use crate::cli::{AltNames, Direction, PinGap, Side};
use core::iter::Iterator;
use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use toml::map::Map;
use toml::Value;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
pub struct Dip {
    pub name: String,               // IC name
    pub title: String,              // IC title
    pub dip: usize,                 // pin count
    pub width: DipWidth,            // package width
    pins: BTreeMap<usize, PinName>, // names of pins
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DipWidth {
    MIL300,
    MIL600,
}

#[derive(Debug, PartialEq)]
struct PinName {
    names: String, // pin names, separated by comma.
}

impl PinName {
    fn names(&self) -> Vec<&str> {
        self.names.split(',').map(str::trim).collect()
    }

    fn name(&self) -> &str {
        self.names().get(0).unwrap()
    }

    fn names_horizontal(&self, names_width: &Vec<usize>, left: bool) -> String {
        let column = names_width.len();
        let names: Vec<&str> = self.names();
        let mut line = String::new();
        for c in 0..column {
            if c != 0 {
                line.push(' ');
            }
            if left {
                let n = column - c - 1;
                if n >= names.len() {
                    line.push_str(&print_spaces(names_width[n]));
                } else {
                    line.push_str(&print_right(names_width[n], names[n]));
                }
            } else {
                if c < names.len() {
                    line.push_str(&print_left(names_width[c], names[c]));
                }
            }
        }

        line
    }

    fn names_vertical(&self, names_width: &Vec<usize>, top: bool) -> Vec<String> {
        let column = names_width.len();
        let names: Vec<&str> = self.names();
        let mut out: Vec<String> = Vec::new();
        for c in 0..column {
            if c != 0 {
                out.push(String::from(" "));
            }
            if top {
                let n = column - 1 - c;
                let text = if n < names.len() { names[n] } else { "" };
                out.append(&mut print_bottom(names_width[n], text));
            } else {
                let text = if c < names.len() { names[c] } else { "" };
                out.append(&mut print_top(names_width[c], text));
            }
        }

        out
    }
}

impl Dip {
    pub fn print(
        &self,
        dir: Direction,
        side: Side,
        show_pin: PinGap,
        show_alt: AltNames,
    ) -> Vec<String> {
        match dir {
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
        }
    }

    fn print_vertical(
        &self,
        side: Side,
        show_pin: PinGap,
        show_alt: AltNames,
        left_start: usize,
        left_end: usize,
        right_start: usize,
        right_end: usize,
    ) -> Vec<String> {
        let (lstart, lend, rstart, rend) = match side {
            Side::TOP => (left_start, left_end, right_start, right_end),
            Side::BOTTOM => (right_start, right_end, left_start, left_end),
        };
        let (lmax, lmaxes) = self.max_name_width(lstart, lend, show_alt);
        let (rmax, rmaxes) = self.max_name_width(rstart, rend, show_alt);
        let (lpin_width, rpin_width) = match show_pin {
            PinGap::PIN2 => (
                self.max_pin_width(lstart, lend) + 2,
                self.max_pin_width(rstart, rend) + 2,
            ),
            PinGap::PIN1 => (
                self.max_pin_width(lstart, lend) + 1,
                self.max_pin_width(rstart, rend) + 1,
            ),
            PinGap::NONE => (0, 0),
        };

        let mut out = Vec::new();
        let mut line = String::new();
        line.push_str(&print_right(lmax + lpin_width + 1, " "));
        line.push_str(&print_chars(self.dip_width(), '_'));
        out.push(line);

        let bottom = self.dip / 2;
        let left = 0;
        let center = self.dip_width() / 2;
        let right = self.dip_width() - 1;
        let name_chars = self.name_chars();
        let num_chars = name_chars.len();
        let name_start = (bottom - num_chars) / 2 + 1;
        let name_end = name_start + num_chars;

        let mut lpin = lstart;
        let mut rpin = rstart;
        for pos in 1..=bottom {
            let mut line = String::new();
            line.push_str(&print_right(
                lmax,
                &self.pin(lpin).names_horizontal(&lmaxes, true),
            ));
            if show_pin != PinGap::NONE {
                line.push_str(&print_right(lpin_width, &lpin.to_string()));
            }

            let spc = if pos == bottom { '_' } else { ' ' };
            line.push('|');
            for c in 0..self.dip_width() {
                let print_name = c == center;
                if lpin == 1 && c == left {
                    line.push('*');
                } else if rpin == 1 && c == right {
                    line.push('*');
                } else if print_name && pos >= name_start && pos < name_end {
                    line += &name_chars[pos - name_start];
                } else {
                    line.push(spc);
                }
            }
            line.push('|');

            if show_pin != PinGap::NONE {
                line.push_str(&print_left(rpin_width, &rpin.to_string()));
            }
            line.push_str(&print_left(
                rmax,
                &self.pin(rpin).names_horizontal(&rmaxes, false),
            ));
            out.push(line);

            lpin = pin_step(lpin, lstart, lend);
            rpin = pin_step(rpin, rstart, rend);
        }
        if show_pin != PinGap::NONE {
            let width = lmax + lpin_width + 1 + (self.dip_width() + self.title.len()) / 2;
            out.push(print_right(width, &self.title));
        }

        out
    }

    fn dip_width(&self) -> usize {
        match self.width {
            DipWidth::MIL300 => 5,
            DipWidth::MIL600 => 9,
        }
    }

    fn print_horizontal(
        &self,
        side: Side,
        show_pin: PinGap,
        show_alt: AltNames,
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
        let width = self.dip / 2;

        let mut out = Vec::new();
        self.print_pins_vertical(tstart, tend, show_pin, show_alt, true, &mut out);
        let center = height / 2 + 1;
        let name_chars = self.name_chars();
        let left = (self.dip - name_chars.len()) / 2;
        let right = left + name_chars.len();
        for l in 1..=height {
            let mut line = String::new();
            let print_name = l == center;
            let mut tpin = tstart;
            let mut bpin = bstart;
            for pos in 1..=width {
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
                    1 if l == 2 && tpin == 1 => "*",
                    1 if l == height - 1 && bpin == 1 => "*",
                    _ if l == 1 || l == height => "-",
                    _ if l == 2 && tpin == 1 => "*",
                    _ if l == height - 1 && bpin == 1 => "*",
                    _ if print_name && c + 1 >= left && c + 1 < right => &name_chars[c + 1 - left],
                    _ => " ",
                });
                tpin = pin_step(tpin, tstart, tend);
                bpin = pin_step(bpin, bstart, bend);
            }
            line.push_str(match l {
                1 => "+",
                _ if l == height => "+",
                _ => "|",
            });
            out.push(line);
        }
        self.print_pins_vertical(bstart, bend, show_pin, show_alt, false, &mut out);

        if show_pin != PinGap::NONE {
            let width = (self.dip + 1 + self.title.len()) / 2;
            out.push(print_right(width, &self.title));
        }

        out
    }

    fn print_pins_vertical(
        &self,
        start: usize,
        end: usize,
        show_pin: PinGap,
        show_alt: AltNames,
        top: bool,
        out: &mut Vec<String>,
    ) {
        let (name_height, names_width) = self.max_name_width(start, end, show_alt);
        let pin_height = self.max_pin_width(start, end);
        let width = self.dip / 2;
        let mut names = vec![String::new(); name_height];
        let mut pins = vec![String::new(); pin_height];
        let mut pin = start;
        if top {
            for _ in 1..=width {
                let name_chars = self.pin(pin).names_vertical(&names_width, top);
                for l in 0..name_height {
                    let line = &mut names[l];
                    line.push(' ');
                    line.push_str(&name_chars[l]);
                }
                if show_pin != PinGap::NONE {
                    let pin_chars = print_bottom(pin_height, &pin.to_string());
                    for l in 0..pin_height {
                        let line = &mut pins[l];
                        line.push(' ');
                        line.push_str(&pin_chars[l]);
                    }
                }
                pin = pin_step(pin, start, end);
            }
            out.append(&mut names);
            if show_pin != PinGap::NONE {
                out.push(String::from(" "));
                out.append(&mut pins);
            }
        } else {
            for _ in 1..=width {
                if show_pin != PinGap::NONE {
                    let pin_chars = print_top(pin_height, &pin.to_string());
                    for l in 0..pin_height {
                        let line = &mut pins[l];
                        line.push(' ');
                        line.push_str(&pin_chars[l]);
                    }
                }
                let name_chars = self.pin(pin).names_vertical(&names_width, top);
                for l in 0..name_height {
                    let line = &mut names[l];
                    line.push(' ');
                    line.push_str(&name_chars[l]);
                }
                pin = pin_step(pin, start, end);
            }
            if show_pin != PinGap::NONE {
                out.append(&mut pins);
                out.push(String::from(" "));
            }
            out.append(&mut names);
        }
    }

    fn dip_height(&self) -> usize {
        match self.width {
            DipWidth::MIL300 => 4,
            DipWidth::MIL600 => 6,
        }
    }

    fn name_chars(&self) -> Vec<String> {
        self.name
            .graphemes(true)
            .map(String::from)
            .collect::<Vec<String>>()
    }

    fn pin(&self, pin_number: usize) -> &PinName {
        &self.pins.get(&pin_number).unwrap()
    }

    fn max_name_width(&self, start: usize, end: usize, show_alt: AltNames) -> (usize, Vec<usize>) {
        let limit = match show_alt {
            AltNames::NONE => 1,
            AltNames::ALT1 => 2,
            AltNames::ALT2 => 3,
            AltNames::ALL => usize::MAX,
        };
        let mut names_width = Vec::new();
        for pin in min(start, end)..=max(start, end) {
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
        let sum: usize = names_width.iter().sum();
        let spaces = names_width.len() - 1;

        (sum + spaces, names_width)
    }

    fn max_pin_width(&self, start: usize, end: usize) -> usize {
        if start < 10 && end < 10 {
            1
        } else {
            2
        }
    }
}

fn pin_step(pin: usize, start: usize, end: usize) -> usize {
    if start < end {
        pin + 1
    } else {
        pin - 1
    }
}

fn print_left(width: usize, text: &str) -> String {
    let mut out = String::from(text);
    if width >= text.len() {
        out.push_str(&print_spaces(width - text.len()));
    }

    out
}

fn print_right(width: usize, text: &str) -> String {
    let mut out = String::new();
    if width >= text.len() {
        out.push_str(&print_spaces(width - text.len()));
    }
    out.push_str(text);

    out
}

#[test]
fn test_print_left_right() {
    assert_eq!(print_left(5, "AB"), "AB   ");
    assert_eq!(print_right(5, "AB"), "   AB");
}

fn print_top(height: usize, text: &str) -> Vec<String> {
    let mut out = text
        .graphemes(true)
        .map(String::from)
        .collect::<Vec<String>>();
    let len = out.len();
    if height > len {
        for _ in 0..(height - len) {
            out.push(String::from(" "));
        }
    }

    out
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

    out
}

#[test]
fn test_print_top_bottom() {
    assert_eq!(print_top(5, "AB"), vec!["A", "B", " ", " ", " "]);
    assert_eq!(print_bottom(5, "AB"), vec![" ", " ", " ", "A", "B"]);
}

fn print_spaces(width: usize) -> String {
    print_chars(width, ' ')
}

fn print_chars(width: usize, c: char) -> String {
    let mut out = String::new();
    for _ in 0..width {
        out.push(c);
    }

    out
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
            Err(err) => Err(err),
            Ok(pins) => Ok(Dip {
                name,
                title,
                dip,
                width,
                pins,
            }),
        }
    }
}

impl FromStr for PinName {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PinName {
            names: s.to_string(),
        })
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
