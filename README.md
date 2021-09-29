[![Rust](https://github.com/tgtakaoka/dip/actions/workflows/rust.yml/badge.svg)](https://github.com/tgtakaoka/dip/actions/workflows/rust.yml)

## DIP (Dual-In-Line Package) pinout drawing

The `dip` command can generate ASCII art DIP package diagram from
simple pin-outs definition in
[TOML](https://github.com/toml-lang/toml).

For example, the following left side TOML file defines
[CD74HCT163E](https://www.ti.com/product/CD74HCT163) _4-Bit Binary
Counter with Synchronous Reset_, and `dip` command will generate the
right side ASCII art diagrams.

```
$ cat cd74hct163.toml    $ dip --pin cd74hct163.toml
name = "74163"                 _____
title = "CD74HCT163"     #MR 1|*    |16 Vcc
dip = 16                  CP 2|  7  |15 TC
width = 300               P0 3|  4  |14 Q0
1 = "#MR"                 P1 4|  1  |13 Q1
2 = "CP"                  P2 5|  6  |12 Q2
3 = "P0"                  P3 6|  3  |11 Q3
4 = "P1"                  PE 7|     |10 TE
5 = "P2"                 GND 8|_____|9  #SPE
6 = "P3"                    CD74HCT163
7 = "PE"                 $ dip --west cd74hct163.toml
8 = "GND"                               #
9 = "#SPE"                V             S
10 = "TE"                 c T Q Q Q Q T P
11 = "Q3"                 c C 0 1 2 3 E E
12 = "Q2"                +---------------+
13 = "Q1"                |               |
14 = "Q0"                |*   74163      |
15 = "TC"                +---------------+
16 = "Vcc"                # C P P P P P G
                          M P 0 1 2 3 E N
                          R             D
```

You can define alternate names for a pin by separating names with a
comma, like so. `dip` can generate bottom-side as well view which can
be handy for soldering.

```
$ grep PB3 atmega328p_megacorex.toml
17 = "PB3, D11, MOSI, OC2A"
$ dip --alt atmega328p_megacorex.toml
               _____
#RESET D22 PC6|*    |PC5  D19 ADC5 SCL 
   RXD  D0 PD0|     |PC4  D18 ADC4 SDA 
   TXD  D1 PD1|  A  |PC3  D17 ADC3     
  INT0  D2 PD2|  T  |PC2  D16 ADC2     
  INT1  D3 PD3|  m  |PC1  D15 ADC1     
        D4 PD4|  e  |PC0  D14 ADC0     
           VCC|  g  |GND               
           GND|  a  |AREF              
 XTAL1 D20 PB6|  3  |AVCC              
 XTAL2 D21 PB7|  2  |PB5  D13 SCK      
  OC0B  D5 PD5|  8  |PB4  D12 MISO     
  OC0A  D6 PD6|  P  |PB3  D11 MOSI OC2A
        D7 PD7|     |PB2  D10 SS   OC1B
        D8 PB0|_____|PB1  D9       OC1A
```

Note that this program is my first [Rust](https://www.rust-lang.org/)
experience. Please let me know if you find better way in Rust in my
code.

```
$ dip --help
dip 0.1.2

USAGE:
    dip [FLAGS] <specifcation_file>

FLAGS:
    -b, --bottom     Bottom-side output
    -e, --east       East direction output
    -h, --help       Prints help information
    -n, --north      North direction output
        --alt        All alternate names output
        --alt1       One alternate name output
        --alt2       Two alternate names output
        --pin        Pin number output with 1 space
        --pin2       Pin number output with 2 spaces
    -s, --south      South direction output
    -t, --top        Top-side output
    -V, --version    Prints version information
    -w, --west       West direction output

ARGS:
    <specifcation_file>    DIP specification file path
```

More information about this command can be found at
https://github.com/tgtakaoka/dip
