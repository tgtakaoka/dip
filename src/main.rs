mod canvas;
mod cli;
mod dip;
mod package;
mod pin;
mod print;
mod quad;

use package::Package;
use std::fs;
use std::process::exit;
use std::str::FromStr;

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

    match Package::from_str(&content) {
        Err(err) => {
            eprintln!("{}", err);
            exit(3);
        }
        Ok(pkg) => {
            for line in pkg.print(args.direction, args.side, args.show_pin, args.show_alt) {
                println!("{}", line);
            }
        }
    }
}
