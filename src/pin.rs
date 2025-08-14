use std::str::FromStr;
use crate::print;

#[derive(Debug, PartialEq)]
pub struct PinName {
    names: String, // pin names, separated by comma.
}

impl PinName {
    pub fn names(&self) -> Vec<&str> {
        self.names.split(',').map(str::trim).collect()
    }

    pub fn name(&self) -> &str {
        self.names().get(0).unwrap()
    }

    pub fn names_horizontal(&self, names_width: &Vec<usize>, left: bool) -> String {
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
                    line.push_str(&print::spaces(names_width[n]));
                } else {
                    line.push_str(&print::right(names_width[n], names[n]));
                }
            } else {
                if c < names.len() {
                    line.push_str(&print::left(names_width[c], names[c]));
                }
            }
        }

        line
    }

    pub fn names_vertical(&self, names_width: &Vec<usize>, top: bool) -> Vec<String> {
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
                out.append(&mut print::bottom(names_width[n], text));
            } else {
                let text = if c < names.len() { names[c] } else { "" };
                out.append(&mut print::top(names_width[c], text));
            }
        }

        out
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
