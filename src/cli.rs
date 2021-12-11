use structopt::StructOpt;

#[derive(Debug)]
pub struct Args {
    pub side: Side,
    pub direction: Direction,
    pub show_pin: PinGap,
    pub show_alt: AltNames,
    pub input: std::path::PathBuf,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Side {
    TOP,
    BOTTOM,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Direction {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PinGap {
    NONE,
    PIN1,
    PIN2,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AltNames {
    NONE,
    ALT1,
    ALT2,
    ALL,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "dip")]
struct Opt {
    /// DIP specification file path
    #[structopt(name = "specifcation_file", parse(from_os_str))]
    input: std::path::PathBuf,
    /// Top-side output
    #[structopt(short = "t", long = "top")]
    top: bool,
    /// Bottom-side output
    #[structopt(short = "b", long = "bottom")]
    bottom: bool,
    /// North direction output
    #[structopt(short = "n", long = "north")]
    north: bool,
    /// East direction output
    #[structopt(short = "e", long = "east")]
    east: bool,
    /// South direction output
    #[structopt(short = "s", long = "south")]
    south: bool,
    /// West direction output
    #[structopt(short = "w", long = "west")]
    west: bool,
    /// Pin number output with 1 space
    #[structopt(long = "pin")]
    pin: bool,
    /// Pin number output with 2 spaces
    #[structopt(long = "pin2")]
    pin2: bool,
    /// All alternate names output
    #[structopt(long = "alt")]
    alt: bool,
    /// One alternate name output
    #[structopt(long = "alt1")]
    alt1: bool,
    /// Two alternate names output
    #[structopt(long = "alt2")]
    alt2: bool,
}

const ERR_SIDE: &str = "Both -t and -b are specified";
const ERR_DIRECTION: &str = "More than one of -n -e -s -w are specified";
const ERR_PIN_NUMBER: &str = "Both --pin and --pin2 are specified";
const ERR_ALT_NAMES: &str = "More than one of --alt --alt1 --alt2 are specified";

fn parse_side(opt: &Opt) -> Result<Side, String> {
    match (opt.top, opt.bottom) {
        (true, true) => Err(ERR_SIDE.to_string()),
        (false, true) => Ok(Side::BOTTOM),
        (_, false) => Ok(Side::TOP),
    }
}

fn parse_direction(opt: &Opt) -> Result<Direction, String> {
    match (opt.north, opt.east, opt.south, opt.west) {
        (_, false, false, false) => Ok(Direction::NORTH),
        (false, true, false, false) => Ok(Direction::EAST),
        (false, false, true, false) => Ok(Direction::SOUTH),
        (false, false, false, true) => Ok(Direction::WEST),
        _ => Err(ERR_DIRECTION.to_string()),
    }
}

fn parse_pins(opt: &Opt) -> Result<PinGap, String> {
    match (opt.pin, opt.pin2) {
        (false, false) => Ok(PinGap::NONE),
        (true, false) => Ok(PinGap::PIN1),
        (false, true) => Ok(PinGap::PIN2),
        _ => Err(ERR_PIN_NUMBER.to_string()),
    }
}

fn parse_alt_names(opt: &Opt) -> Result<AltNames, String> {
    match (opt.alt, opt.alt1, opt.alt2) {
        (false, false, false) => Ok(AltNames::NONE),
        (true, false, false) => Ok(AltNames::ALL),
        (false, true, false) => Ok(AltNames::ALT1),
        (false, false, true) => Ok(AltNames::ALT2),
        _ => Err(ERR_ALT_NAMES.to_string()),
    }
}

pub fn parse_args() -> Result<Args, String> {
    let opt = Opt::from_args();
    Ok(Args {
        side: parse_side(&opt)?,
        direction: parse_direction(&opt)?,
        show_pin: parse_pins(&opt)?,
        show_alt: parse_alt_names(&opt)?,
        input: opt.input,
    })
}
