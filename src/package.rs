use crate::cli::{AltNames, Direction, PinGap, Side};
use crate::dip::Dip;
use crate::quad::Quad;
use std::str::FromStr;

pub enum Package {
    Dip(Dip),
    Quad(Quad),
}

impl Package {
    pub fn print(
        &self,
        dir: Direction,
        side: Side,
        show_pin: PinGap,
        show_alt: AltNames,
    ) -> Vec<String> {
        match self {
            Package::Dip(d) => d.print(dir, side, show_pin, show_alt),
            Package::Quad(q) => q.print(dir, side, show_pin, show_alt),
        }
    }
}

impl FromStr for Package {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match toml::from_str::<toml::Value>(s.trim()) {
            Err(err) => return Err(err.to_string()),
            Ok(p) => p,
        };
        let table = v.as_table().unwrap();
        let has_dip = table.contains_key("dip") || table.contains_key("sdip");
        let has_quad =
            table.contains_key("plcc") || table.contains_key("qfp") || table.contains_key("bqfp");

        match (has_dip, has_quad) {
            (true, true) => Err("cannot specify both dip/sdip and plcc/qfp/bqfp".to_string()),
            (false, false) => {
                Err("no dip, sdip, plcc, qfp, or bqfp package field".to_string())
            }
            (true, false) => Dip::from_str(s).map(Package::Dip),
            (false, true) => Quad::from_str(s).map(Package::Quad),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse `"-- <flags>"` from a fixture case header into (Direction, Side, PinGap, AltNames).
    fn parse_case_args(args: &str) -> (Direction, Side, PinGap, AltNames) {
        let mut dir = Direction::North;
        let mut side = Side::Top;
        let mut pin = PinGap::None;
        let mut alt = AltNames::None;
        for token in args.split_whitespace() {
            match token {
                "--north"  => dir  = Direction::North,
                "--south"  => dir  = Direction::South,
                "--east"   => dir  = Direction::East,
                "--west"   => dir  = Direction::West,
                "--top"    => side = Side::Top,
                "--bottom" => side = Side::Bottom,
                "--pin"    => pin  = PinGap::Pin1,
                "--alt"    => alt  = AltNames::All,
                "--alt1"   => alt  = AltNames::Alt1,
                "--alt2"   => alt  = AltNames::Alt2,
                // bare direction words without --
                "north"    => dir  = Direction::North,
                "south"    => dir  = Direction::South,
                "east"     => dir  = Direction::East,
                "west"     => dir  = Direction::West,
                _          => {}
            }
        }
        (dir, side, pin, alt)
    }

    /// Run all golden cases in a fixture file.
    ///
    /// Format:
    ///   <TOML input lines>
    ///
    ///   --- [args]
    ///   <expected ASCII art lines>
    ///
    ///   --- [args]
    ///   ...
    fn run_golden(src: &str) {
        // Split on lines starting with "--- "  or exactly "---"
        let mut sections: Vec<(Option<&str>, Vec<&str>)> = Vec::new();
        let mut current_header: Option<&str> = None;
        let mut current_lines: Vec<&str> = Vec::new();

        for line in src.lines() {
            if let Some(rest) = line.strip_prefix("---") {
                sections.push((current_header, current_lines.clone()));
                current_header = Some(rest.trim());
                current_lines.clear();
            } else {
                current_lines.push(line);
            }
        }
        sections.push((current_header, current_lines));

        // First section (header=None) is the TOML
        let toml_src = sections[0].1.join("\n");
        let pkg = Package::from_str(&toml_src)
            .unwrap_or_else(|e| panic!("TOML parse error: {}", e));

        // Remaining sections are cases
        for (header, raw_lines) in &sections[1..] {
            let args_str = header.unwrap_or("");
            let (dir, side, pin, alt) = parse_case_args(args_str);

            // Strip leading and trailing blank lines from expected output
            let expected: Vec<&str> = {
                let trimmed: Vec<&str> = raw_lines
                    .iter()
                    .copied()
                    .skip_while(|l| l.trim().is_empty())
                    .collect();
                let end = trimmed
                    .iter()
                    .rposition(|l| !l.trim().is_empty())
                    .map(|i| i + 1)
                    .unwrap_or(0);
                trimmed[..end].to_vec()
            };

            let actual: Vec<String> = pkg.print(dir, side, pin, alt);
            let actual_refs: Vec<&str> = actual.iter().map(String::as_str).collect();

            /*
            assert_eq!(actual_refs.len(), expected.len());
            for (index, actual_row) in actual_refs.iter().enumerate() {
                let expected_row = &expected[index];
                assert_eq!(actual_row, expected_row,
                    "\nCase {:?} row {:?} failed.\nExpected: {}\n  Actual: {}",
                    args_str, index, expected_row, actual_row,
                );                    
            }
            */

            assert_eq!(
                actual_refs, expected,
                "\nCase {:?} failed.\nExpected:\n{}\nActual:\n{}",
                args_str,
                expected.iter().map(|s| format!("'{}'\n", s)).collect::<String>(),
                actual_refs.iter().map(|s| format!("'{}'\n", s)).collect::<String>(),
            );
        }
    }

    #[test]
    fn golden_dip20() {
        run_golden(include_str!("../tests/data/dip20.txt"));
    }

    #[test]
    fn golden_dip40() {
        run_golden(include_str!("../tests/data/dip40.txt"));
    }

    #[test]
    fn golden_plcc44_tc() {
        run_golden(include_str!("../tests/data/plcc44_tc.txt"));
    }

    #[test]
    fn golden_plcc44_lt() {
        run_golden(include_str!("../tests/data/plcc44_lt.txt"));
    }

    #[test]
    fn golden_qfp44() {
        run_golden(include_str!("../tests/data/qfp44.txt"));
    }

    #[test]
    fn golden_qfp_rect() {
        run_golden(include_str!("../tests/data/qfp_rect.txt"));
    }

    #[test]
    fn golden_bqfp44() {
        run_golden(include_str!("../tests/data/bqfp44.txt"));
    }
}
