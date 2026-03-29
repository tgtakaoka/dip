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
$ cat samples/cd74hct163.toml
name = "74163"
title = "CD74HCT163"
dip = 16
width = 300
1 = "#MR"
2 = "CP"
3 = "P0"
4 = "P1"
5 = "P2"
6 = "P3"
7 = "PE"
8 = "GND"
9 = "#SPE"
10 = "TE"
11 = "Q3"
12 = "Q2"
13 = "Q1"
14 = "Q0"
15 = "TC"
16 = "Vcc"
```
```
$ dip --pin samples/cd74hct163.toml
               #
 V             S
 c T Q Q Q Q T P
 c C 0 1 2 3 E E

 1 1 1 1 1 1 1
 6 5 4 3 2 1 0 9
+---------------+
|               |
|* CD74HCT163   |
+---------------+
 1 2 3 4 5 6 7 8

 # C P P P P P G
 M P 0 1 2 3 E N
 R             D
   CD74HCT163
```
```
$ dip --east samples/cd74hct163.toml
    +------+
#MR |*     | Vcc
 CP |   7  | TC
 P0 |   4  | Q0
 P1 |   1  | Q1
 P2 |   6  | Q2
 P3 |   3  | Q3
 PE |      | TE
GND |      | #SPE
    +------+
```

### PLCC/QFP example

Quad-style packages use `plcc = N`, `qfp = N`, or `bqfp = N` (or an
asymmetric `"NxM"` string for QFP/BQFP).  Pin 1 position is
`top-center` by default; use `pin1 = "left-top"` or
`pin1 = "bottom-left"` when needed.

```
$ dip --pin samples/n80186.toml
                                   C
                                 R L
                   #             E K A
           A A A A B # # A V     S O R # # #
           1 1 1 1 H W R L S X X E U D S S S
           6 7 8 9 E R D E S 1 2 T T Y 2 1 0

           6 6 6 6 6 6 6 6 6 5 5 5 5 5 5 5 5
           8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2
          /----------------------------------+
AD15   1 |*----------------------------------| 51  HLDA
 AD7   2 |                                   | 50  HOLD
AD14   3 |                                   | 49  SRDY
 AD6   4 |                                   | 48  #LOCK
AD13   5 |                                   | 47  #TEST
 AD5   6 |                                   | 46  NMI
AD12   7 |                                   | 45  INT0
 AD4   8 |                                   | 44  INT1
 VCC   9 |              N80186               | 43  VCC
AD10  10 |                                   | 42  INT2
 AD3  11 |                                   | 41  INT3
AD10  12 |                                   | 40  DT/#R
 AD2  13 |                                   | 39  #DEN
 AD9  14 |                                   | 38  #MCS0
 AD1  15 |                                   | 37  #MCS1
 AD8  16 |                                   | 36  #MCS2
 AD0  17 |                                   | 35  #MCS3
         +-----------------------------------+
           1 1 2 2 2 2 2 2 2 2 2 2 3 3 3 3 3
           8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4

           D D T T T T # # V # # # # # # # #
           R R M M M M R P S P P P P P P L U
           Q Q R R R R E C S C C C C C C C C
           0 1 I I O O S S   S S S S S S S S
               N N U U   0   1 2 3 4 5 6
               0 1 T T
                   0 1
                         N80186
```
```
$ dip --pin samples/z80180fsc.toml
                      # #
              #     # T D                       # # #
              R ( ( H E R       C R T T ( C R T D C R   ( (
              F n n A N E C R T K X E X n K X X C T T   n n
              S c c L D Q K X X A A S A c A A A D S S D c c D
              H ) ) T 1 1 S S S 1 1 T 1 ) 0 0 0 0 0 0 7 ) ) 6

              6 6 6 6 6 5 5 5 5 5 5 5 5 5 5 4 4 4 4 4 4 4 4 4
              4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1
            +-------------------------------------------------+
  #IORQ  65 |                                                 | 40  D5
  #MREQ  66 |                                                 | 39  D4
      E  67 |                                                 | 38  D3
    #M1  68 |                                                 | 37  D2
    #WR  69 |                                                 | 36  D1
    #RD  70 |                                                 | 35  D0
    PHI  71 |                                                 | 34  VSS
    VSS  72 |                                                 | 33  A19
    CSS  73 |                    Z80180FSC                    | 32  VCC
   XTAL  74 |                                                 | 31  A18
   (nc)  75 |                                                 | 30  (nc)
  EXTAL  76 |                                                 | 29  A17
  #WAIT  77 |                                                 | 28  A16
#BUSACK  78 |                                                 | 27  A15
#BUSREQ  79 |                                                 | 26  A14
 #RESET  80 | *                                               | 25  A13
            +-------------------------------------------------+
              1 2 3 4 5 6 7 8 9 1 1 1 1 1 1 1 1 1 1 2 2 2 2 2
                                0 1 2 3 4 5 6 7 8 9 0 1 2 3 4

              # ( ( # # # S A A A A V A ( A A A A A A A ( ( A
              N n n I I I T 0 1 2 3 S 4 n 5 6 7 8 9 1 1 n n 1
              M c c N N N           S   c           0 1 c c 2
              I ) ) T T T               )               ) )
                    0 1 2
                               Z80180FSC
```

### Alternate pin names

Pin entries may contain comma-separated alternate names.  Use
`--alt` / `--alt1` / `--alt2` to reveal them:

```
$ grep PB3 atmega328p_minicore.toml
17 = "PB3, D11, MOSI, OC2A"
$ dip --alt --pin --east samples/atmega328p_minicore.toml
                   +------+
#RESET D22 PC6   1 |*     |28  PC5  D19 ADC5 SCL
   RXD  D0 PD0   2 |      |27  PC4  D18 ADC4 SDA
   TXD  D1 PD1   3 |   A  |26  PC3  D17 ADC3
  INT0  D2 PD2   4 |   T  |25  PC2  D16 ADC2
  INT1  D3 PD3   5 |   m  |24  PC1  D15 ADC1
        D4 PD4   6 |   e  |23  PC0  D14 ADC0
           VCC   7 |   g  |22  GND
           GND   8 |   a  |21  AREF
 XTAL1 D20 PB6   9 |   3  |20  AVCC
 XTAL2 D21 PB7  10 |   2  |19  PB5  D13 SCK
  OC0B  D5 PD5  11 |   8  |18  PB4  D12 MISO
  OC0A  D6 PD6  12 |   P  |17  PB3  D11 MOSI OC2A
        D7 PD7  13 |      |16  PB2  D10 SS   OC1B
        D8 PB0  14 |      |15  PB1  D9       OC1A
                   +------+
```

### Viewing from below

`--bottom` mirrors the diagram as if viewed from the solder side:

```
$ dip --bottom --east --pin samples/cd74hct163.toml
         +------+
 Vcc  16 |     *|1   #MR
  TC  15 |   7  |2   CP
  Q0  14 |   4  |3   P0
  Q1  13 |   1  |4   P1
  Q2  12 |   6  |5   P2
  Q3  11 |   3  |6   P3
  TE  10 |      |7   PE
#SPE   9 |      |8   GND
         +------+
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

More information: https://github.com/tgtakaoka/dip
