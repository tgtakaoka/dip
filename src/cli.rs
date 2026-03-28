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
    Top,
    Bottom,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PinGap {
    None,
    Pin1,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AltNames {
    None,
    Alt1,
    Alt2,
    All,
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
    /// Pin number output
    #[structopt(long = "pin")]
    pin: bool,
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
const ERR_ALT_NAMES: &str = "More than one of --alt --alt1 --alt2 are specified";

fn parse_side(opt: &Opt) -> Result<Side, String> {
    match (opt.top, opt.bottom) {
        (true, true) => Err(ERR_SIDE.to_string()),
        (false, true) => Ok(Side::Bottom),
        (_, false) => Ok(Side::Top),
    }
}

fn parse_direction(opt: &Opt) -> Result<Direction, String> {
    match (opt.north, opt.east, opt.south, opt.west) {
        (_, false, false, false) => Ok(Direction::North),
        (false, true, false, false) => Ok(Direction::East),
        (false, false, true, false) => Ok(Direction::South),
        (false, false, false, true) => Ok(Direction::West),
        _ => Err(ERR_DIRECTION.to_string()),
    }
}

fn parse_pins(opt: &Opt) -> Result<PinGap, String> {
    if opt.pin { Ok(PinGap::Pin1) } else { Ok(PinGap::None) }
}

fn parse_alt_names(opt: &Opt) -> Result<AltNames, String> {
    match (opt.alt, opt.alt1, opt.alt2) {
        (false, false, false) => Ok(AltNames::None),
        (true, false, false) => Ok(AltNames::All),
        (false, true, false) => Ok(AltNames::Alt1),
        (false, false, true) => Ok(AltNames::Alt2),
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
