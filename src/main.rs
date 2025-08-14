mod cli;
mod dip;
mod pin;
mod print;

use dip::Dip;
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

    match Dip::from_str(&content) {
        Err(err) => {
            eprintln!("{}", err);
            exit(3);
        }
        Ok(dip) => {
            for line in dip.print(args.direction, args.side, args.show_pin, args.show_alt) {
                println!("{}", line);
            }
        }
    }
}
