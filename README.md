[![Release](https://img.shields.io/github/v/release/tgtakaoka/dip.svg?maxAge=3600)](https://github.com/tgtakaoka/dip/releases)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/tgtakaoka/dip/blob/main/LICENSE.md)
[![Rust](https://github.com/tgtakaoka/dip/actions/workflows/rust.yml/badge.svg)](https://github.com/tgtakaoka/dip/actions/workflows/rust.yml)

## DIP/PLCC/QFP pinout drawing

The `dip` command generates ASCII art package diagrams from simple
pin-out definitions in [TOML](https://github.com/toml-lang/toml).

Supported package types:

- **DIP / SDIP** — Dual Inline Package
- **PLCC** — Plastic Leaded Chip Carrier
- **QFP** — Quad Flat Package (including asymmetric `NxM` pin counts)
- **BQFP** — Bumpered Quad Flat Package

### DIP example

The following TOML defines
[CD74HCT163E](https://www.ti.com/product/CD74HCT163)
_4-Bit Binary Counter with Synchronous Reset_.

```
$ cat cd74hct163.toml       $ dip --pin cd74hct163.toml
name = "74163"                             #
title = "CD74HCT163"          V             S
dip = 16                      c T Q Q Q Q T P
width = 300                   c C 0 1 2 3 E E
1 = "#MR"
2 = "CP"                      1 1 1 1 1 1 1
3 = "P0"                      6 5 4 3 2 1 0 9
4 = "P1"                     +---------------+
5 = "P2"                     |               |
6 = "P3"                     |* CD74HCT163   |
7 = "PE"                     +---------------+
8 = "GND"                     1 2 3 4 5 6 7 8
9 = "#SPE"
10 = "TE"                     # C P P P P P G
11 = "Q3"                     M P 0 1 2 3 E N
12 = "Q2"                     R             D
13 = "Q1"                       CD74HCT163
14 = "Q0"
15 = "TC"                    $ dip --west cd74hct163.toml
16 = "Vcc"                        +------+
                               #SPE|      | GND
                                 TE|   7  | PE
                                 Q3|   4  | P3
                                 Q2|   1  | P2
                                 Q1|   6  | P1
                                 Q0|   3  | P0
                                 TC|      | CP
                                Vcc|     *| #MR
                                   +------+
```

### PLCC/QFP example

Quad-style packages use `plcc = N`, `qfp = N`, or `bqfp = N` (or an
asymmetric `"NxM"` string for QFP/BQFP).  Pin 1 position is
`top-center` by default; use `pin1 = "left-top"` or
`pin1 = "bottom-left"` when needed.

```
$ dip samples/n80186.toml | head -12
                               C
                             R L
               #             E K A
       A A A A B # # A V     S O R # # #
       1 1 1 1 H W R L S X X E U D S S S
       6 7 8 9 E R D E S 1 2 T T Y 2 1 0
     +----------------------------------+
 AD15|*                                 | HLDA
  AD7|                                  | HOLD
 AD14|                                  | SRDY
  AD6|                                  | #LOCK
```

### Alternate pin names

Pin entries may contain comma-separated alternate names.  Use
`--alt` / `--alt1` / `--alt2` to reveal them:

```
$ grep PB3 atmega328p_minicore.toml
17 = "PB3, D11, MOSI, OC2A"
$ dip --alt --pin --west samples/atmega328p_minicore.toml | head -5
                      +------+
OC1A       D9  PB1  15|      |14  PB0 D8
OC1B   SS D10  PB2  16|      |13  PD7 D7
OC2A MOSI D11  PB3  17|   A  |12  PD6 D6  OC0A
     MISO D12  PB4  18|   T  |11  PD5 D5  OC0B
```

### Viewing from below

`--bottom` mirrors the diagram as if viewed from the solder side:

```
$ dip --bottom --pin cd74hct163.toml
```

### Usage

```
$ dip --help
dip 0.2.0

USAGE:
    dip [FLAGS] <specifcation_file>

FLAGS:
        --alt        All alternate names output
        --alt1       One alternate name output
        --alt2       Two alternate names output
    -b, --bottom     Bottom-side output
    -e, --east       East direction output
    -h, --help       Prints help information
    -n, --north      North direction output
        --pin        Pin number output
    -s, --south      South direction output
    -t, --top        Top-side output
    -V, --version    Prints version information
    -w, --west       West direction output

ARGS:
    <specifcation_file>    DIP specification file path
```

Note that this program started as my first [Rust](https://www.rust-lang.org/)
project.  Feedback and suggestions for more idiomatic Rust are welcome.

More information: https://github.com/tgtakaoka/dip
