use structopt::StructOpt;

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
    #[structopt(long = "pin",)]
    show_pin: bool,
    /// Pin number output with 2 spaces
    #[structopt(long = "pin2")]
    show_pin2: bool,
    /// All alternate names output
    #[structopt(long = "alt")]
    show_alt: bool,
    /// One alternate name output
    #[structopt(long = "alt1")]
    show_alt1: bool,
    /// Two alternate names output
    #[structopt(long = "alt2")]
    show_alt2: bool,
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

#[derive(Debug)]
pub struct Args {
    pub side: Side,
    pub direction: Direction,
    pub show_pin: PinGap,
    pub show_alt: AltNames,
    pub input: std::path::PathBuf,
}

const ERR_SIDE: &str = "Both -t and -b are specified";
const ERR_DIRECTION: &str = "More than one of -n -e -s -w are specified";
const ERR_PIN_NUMBER: &str = "Both --pin and --pin2 are specified";
const ERR_ALT_NAMES: &str = "More than one of --alt --alt1 --alt2 are specified";

fn parse_side(top: bool, bottom: bool) -> Result<Side, String> {
    if top && bottom {
        return Err(ERR_SIDE.to_string());
    }
    return Ok(if bottom { Side::BOTTOM } else { Side::TOP });
}

#[test]
fn test_side() {
    assert_eq!(Ok(Side::TOP), parse_side(false, false));
    assert_eq!(Ok(Side::TOP), parse_side(true, false));
    assert_eq!(Ok(Side::BOTTOM), parse_side(false, true));
    assert_eq!(Err(ERR_SIDE.to_string()), parse_side(true, true));
}

fn parse_direction(north: bool, east: bool, south: bool, west: bool) -> Result<Direction, String> {
    if (north && (east || south || west)) || (east && (south || west)) || (south && west) {
        return Err(ERR_DIRECTION.to_string());
    }
    return Ok(if east {
        Direction::EAST
    } else if west {
        Direction::WEST
    } else if south {
        Direction::SOUTH
    } else {
        Direction::NORTH
    });
}

#[test]
fn test_direction() {
    assert_eq!(
        Ok(Direction::NORTH),
        parse_direction(false, false, false, false)
    );
    assert_eq!(
        Ok(Direction::NORTH),
        parse_direction(true, false, false, false)
    );
    assert_eq!(
        Ok(Direction::EAST),
        parse_direction(false, true, false, false)
    );
    assert_eq!(
        Ok(Direction::SOUTH),
        parse_direction(false, false, true, false)
    );
    assert_eq!(
        Ok(Direction::WEST),
        parse_direction(false, false, false, true)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(true, true, false, false)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(true, false, true, false)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(true, false, false, true)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(false, true, true, false)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(false, true, false, true)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(false, false, true, true)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(true, true, true, false)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(true, true, false, true)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(true, false, true, true)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(false, true, true, true)
    );
    assert_eq!(
        Err(ERR_DIRECTION.to_string()),
        parse_direction(true, true, true, true)
    );
}

fn parse_pins(pin: bool, pin2: bool) -> Result<PinGap, String> {
    if pin && pin2 {
        return Err(ERR_PIN_NUMBER.to_string());
    }
    if pin {
        return Ok(PinGap::PIN1);
    } else if pin2 {
        return Ok(PinGap::PIN2);
    } else {
        return Ok(PinGap::NONE);
    }
}

fn parse_alt_names(alt: bool, alt1: bool, alt2: bool) -> Result<AltNames, String> {
    if (alt && (alt1 || alt2)) || (alt1 && alt2) {
        return Err(ERR_ALT_NAMES.to_string());
    }
    if alt {
        return Ok(AltNames::ALL);
    } else if alt1 {
        return Ok(AltNames::ALT1);
    } else if alt2 {
        return Ok(AltNames::ALT2);
    } else {
        return Ok(AltNames::NONE);
    }
}

pub fn parse_args() -> Result<Args, String> {
    let opt = Opt::from_args();
    let mut args = Args {
        side: Side::TOP,
        direction: Direction::NORTH,
        input: opt.input,
        show_pin: PinGap::NONE,
        show_alt: AltNames::NONE,
    };
    args.side = match parse_side(opt.top, opt.bottom) {
        Err(err) => return Err(err),
        Ok(side) => side,
    };
    args.direction = match parse_direction(opt.north, opt.east, opt.south, opt.west) {
        Err(err) => return Err(err),
        Ok(direction) => direction,
    };
    args.show_pin = match parse_pins(opt.show_pin, opt.show_pin2) {
        Err(err) => return Err(err),
        Ok(show_pin) => show_pin,
    };
    args.show_alt = match parse_alt_names(opt.show_alt, opt.show_alt1, opt.show_alt2) {
        Err(err) => return Err(err),
        Ok(show_alt) => show_alt,
    };
    return Ok(args);
}
