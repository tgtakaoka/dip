// PLCC (Plastic Leaded Chip Carrier) package implementation
// Pin 1 is in the middle of the top edge when notch is at top-left
// Pins number counter-clockwise from pin 1

extern crate unicode_segmentation;

use crate::cli::{AltNames, Direction, PinGap, Side};
use crate::package::Package;
use crate::pin::PinName;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use toml::Table;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
pub struct Plcc {
    pub name: String,
    pub title: String,
    pub pins: usize,
    pub width: PlccWidth,
    pins_map: BTreeMap<usize, PinName>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlccWidth {
    MIL450,   // 18-pin, 20-pin
    MIL550,   // 28-pin
    MIL650,   // 32-pin
    MIL850,   // 44-pin
    MIL1150,  // 52-pin, 68-pin, 84-pin
}

impl Plcc {
    fn pins_per_side(&self) -> usize {
        self.pins / 4
    }

    fn package_width(&self) -> usize {
        match self.width {
            PlccWidth::MIL450 => 11,
            PlccWidth::MIL550 => 13,
            PlccWidth::MIL650 => 15,
            PlccWidth::MIL850 => 19,
            PlccWidth::MIL1150 => 25,
        }
    }

    fn package_height(&self) -> usize {
        self.package_width()
    }

    fn pin(&self, pin_number: usize) -> &PinName {
        self.pins_map.get(&pin_number).unwrap()
    }

    fn name_chars(&self) -> Vec<String> {
        self.name
            .graphemes(true)
            .map(String::from)
            .collect::<Vec<String>>()
    }

    pub fn print(
        &self,
        _dir: Direction,
        _side: Side,
        _show_pin: PinGap,
        _show_alt: AltNames,
    ) -> Vec<String> {
        let mut out = Vec::new();
        let width = self.package_width();
        
        // Simple placeholder visualization
        out.push(format!("+{}+", "-".repeat(width - 2)));
        out.push(format!("|{:^width$}|", self.name, width = width - 2));
        out.push(format!("|{:^width$}|", format!("PLCC-{}", self.pins), width = width - 2));
        out.push(format!("+{}+", "-".repeat(width - 2)));
        
        out
    }
}

impl Package for Plcc {
    fn name(&self) -> &str {
        &self.name
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn pin_count(&self) -> usize {
        self.pins
    }

    fn print(
        &self,
        dir: Direction,
        side: Side,
        show_pin: PinGap,
        show_alt: AltNames,
    ) -> Vec<String> {
        self.print(dir, side, show_pin, show_alt)
    }
}

impl fmt::Display for Plcc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[name={} ", self.name)?;
        write!(f, "title={} ", self.title)?;
        write!(f, "package=PLCC{} width={:?} ", self.pins, self.width)?;
        write!(f, "pins=[");
        for pin in 1..self.pins {
            write!(f, "{} ", self.pin(pin).name())?;
        }
        write!(f, "{}]]", self.pin(self.pins).name())
    }
}

impl FromStr for Plcc {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let toml = s.trim().parse::<Table>().map_err(|err| err.to_string())?;

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

        let pins = match toml.get("pins") {
            None => return Err("no pins count".to_string()),
            Some(v) => match v.as_integer() {
                None => return Err("pins must be number".to_string()),
                Some(n) if n <= 0 => return Err(format!("pins {} must be positive", n)),
                Some(n) if n >= 200 => return Err(format!("pins {} must be less than 200", n)),
                Some(n) if n % 4 != 0 => return Err(format!("PLCC pins {} must be divisible by 4", n)),
                Some(n) => usize::try_from(n).ok().unwrap(),
            },
        };

        let width = match toml.get("width") {
            None => return Err("no width".to_string()),
            Some(v) => match v.as_integer() {
                None => return Err("width must be number in mil".to_string()),
                Some(w) if w == 450 => PlccWidth::MIL450,
                Some(w) if w == 550 => PlccWidth::MIL550,
                Some(w) if w == 650 => PlccWidth::MIL650,
                Some(w) if w == 850 => PlccWidth::MIL850,
                Some(w) if w == 1150 => PlccWidth::MIL1150,
                Some(w) => return Err(format!("unknown PLCC width {}", w)),
            },
        };

        match pins_to_map(&toml, pins) {
            Err(err) => Err(err),
            Ok(pins_map) => Ok(Plcc {
                name,
                title,
                pins,
                width,
                pins_map,
            }),
        }
    }
}

fn pins_to_map(
    toml: &Table,
    pin_count: usize,
) -> Result<BTreeMap<usize, PinName>, String> {
    let mut pins = BTreeMap::new();
    for pin in toml.keys() {
        if let Ok(n) = pin.parse::<usize>() {
            if n == 0 {
                return Err("invalid pin number 0".to_string());
            }
            if n > pin_count {
                return Err(format!(
                    "pin number {} must not be greater than pins {}",
                    n, pin_count
                ));
            }
            match toml.get(pin).unwrap().as_str() {
                None => return Err(format!("name for pin {} must be string", n)),
                Some(name) => pins.insert(n, PinName::from_str(name).unwrap()),
            };
        }
    }

    for p in 1..=pin_count {
        if !pins.contains_key(&p) {
            return Err(format!("missing pin {} definition", p));
        }
    }

    Ok(pins)
}
