mod cli;
mod package;
mod dip;
mod qfp;
mod plcc;
mod pin;
mod print;

use dip::Dip;
use package::Package;
use qfp::Qfp;
use plcc::Plcc;
use std::fs;
use std::process::exit;
use std::str::FromStr;
use toml::Table;

fn detect_package_type(content: &str) -> Result<String, String> {
    let toml = content.trim().parse::<Table>().map_err(|err| err.to_string())?;
    
    // Check for explicit "package" field
    if let Some(pkg) = toml.get("package") {
        if let Some(pkg_str) = pkg.as_str() {
            return Ok(pkg_str.to_string());
        }
    }
    
    // Fall back to checking for package-specific fields for backward compatibility
    if toml.contains_key("dip") {
        return Ok("dip".to_string());
    }
    if toml.contains_key("pins") {
        // If pins field exists but no dip field, assume QFP
        return Ok("qfp".to_string());
    }
    
    Err("Cannot determine package type. Please specify 'package = \"dip\"', 'package = \"qfp\"', or 'package = \"plcc\"'".to_string())
}

fn main() {
    let args_result = cli::parse_args();
    let content = match &args_result {
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
        Ok(args) => match fs::read_to_string(&args.input) {
            Ok(content) => content,
            Err(_err) => {
                eprintln!("can't open {:?}", &args.input);
                exit(2);
            }
        },
    };
    let args = args_result.unwrap();

    let package_type = match detect_package_type(&content) {
        Ok(pt) => pt,
        Err(err) => {
            eprintln!("{}", err);
            exit(3);
        }
    };

    let pkg: Box<dyn Package> = match package_type.as_str() {
        "dip" => match Dip::from_str(&content) {
            Ok(dip) => Box::new(dip),
            Err(err) => {
                eprintln!("{}", err);
                exit(4);
            }
        },
        "qfp" => match Qfp::from_str(&content) {
            Ok(qfp) => Box::new(qfp),
            Err(err) => {
                eprintln!("{}", err);
                exit(4);
            }
        },
        "plcc" => match Plcc::from_str(&content) {
            Ok(plcc) => Box::new(plcc),
            Err(err) => {
                eprintln!("{}", err);
                exit(4);
            }
        },
        _ => {
            eprintln!("Unknown package type: {}", package_type);
            exit(3);
        }
    };

    for line in pkg.print(args.direction, args.side, args.show_pin, args.show_alt) {
        println!("{}", line);
    }
}
