mod cli;
mod dip;

use dip::Dip;

fn main() {
    match cli::parse_args() {
        Err(err) => eprintln!("{}", err),
        Ok(args) => {
            println!("input {:?}", args);
            let mut chip = Dip::new("MC6809E");
            chip.add(1, "#RESET");
            println!("name:{:?} pins:{:?}", chip.name, chip.pins);
        }
    }
}
