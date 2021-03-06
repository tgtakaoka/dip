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
}

#[derive(PartialEq, Debug)]
pub enum Side {
    TOP,
    BOTTOM,
}

#[derive(PartialEq, Debug)]
pub enum Direction {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

#[derive(Debug)]
pub struct Args {
    side: Side,
    direction: Direction,
    input: std::path::PathBuf,
}

const ERR_SIDE: &str = "Both -t and -b are specified";
const ERR_DIRECTION: &str = "More than one of -n -e -s -w are specified";

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

pub fn parse_args() -> Result<Args, String> {
    let opt = Opt::from_args();
    let mut args = Args {
        side: Side::TOP,
        direction: Direction::NORTH,
        input: opt.input,
    };
    args.side = match parse_side(opt.top, opt.bottom) {
        Err(err) => return Err(err),
        Ok(side) => side,
    };
    args.direction = match parse_direction(opt.north, opt.east, opt.south, opt.west) {
        Err(err) => return Err(err),
        Ok(direction) => direction,
    };
    return Ok(args);
}
