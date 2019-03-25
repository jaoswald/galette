use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

const INPUT: i32 = 2;

fn make_spaces(buf: &mut String, n: usize) {
    for _i in 0..n {
        buf.push(' ');
    }
}

fn make_chip(gal_type: i32, pin_names: &[&str]) -> String {
    let num_of_pins = pin_names.len();
    let mut buf = String::new();

    buf.push_str("\n\n");

    make_spaces(&mut buf, 31);

    buf.push_str(match gal_type {
        ::interop::GAL16V8   => " GAL16V8\n\n",
        ::interop::GAL20V8   => " GAL20V8\n\n",
        ::interop::GAL22V10  => " GAL22V10\n\n",
        ::interop::GAL20RA10 => "GAL20RA10\n\n",
        _ => panic!("Nope"),
    });

    make_spaces(&mut buf, 26);

    buf.push_str("-------\\___/-------\n");

    let mut started = false;
    for n in 0..num_of_pins / 2 {
        if started {
            make_spaces(&mut buf, 26);
            buf.push_str("|                 |\n");
        } else {
            started = true;
        }

        make_spaces(&mut buf, 25 - pin_names[n].len());

        buf.push_str(&format!("{} | {:>2}           {:>2} | {}\n",
                     pin_names[n],
                     n + 1,
                     num_of_pins - n,
                     pin_names[num_of_pins - n - 1]));
    }

    make_spaces(&mut buf, 26);
    buf.push_str("-------------------\n");

    return buf;
}

const DUMMY_OLMC12: usize = 25;

fn is_olmc(gal_type: i32, n: usize) -> bool {
    match gal_type {
    ::interop::GAL16V8 => n >= 12 && n <= 19,
    ::interop::GAL20V8 => n >= 15 && n <= 22,
    ::interop::GAL22V10 => n >= 14 && n <= DUMMY_OLMC12,
    ::interop::GAL20RA10 => n >= 14 && n <= 23,
    _ => panic!("Nope"),
    }
}

fn pin_to_olmc(gal_type: i32, pin: usize) -> usize {
    pin - match gal_type {
        ::interop::GAL16V8 => 12,
        ::interop::GAL20V8 => 15,
        ::interop::GAL22V10 => 14,
        ::interop::GAL20RA10 => 14,
        _ => panic!("Nope")
    }
}

fn make_pin(gal_type: i32, pin_names: &[&str], mode: i32, olmc_pin_types: &[i32]) -> String {
    let num_of_pins = pin_names.len();

    let mut buf = String::new();
    buf.push_str("\n\n");
    buf.push_str(" Pin # | Name     | Pin Type\n");
    buf.push_str("-----------------------------\n");

    for n in 1..num_of_pins + 1 {
        buf.push_str(&format!("  {:>2}   | ", n));
        buf.push_str(pin_names[n - 1]);

        make_spaces(&mut buf, 9 - pin_names[n-1].len());

        let mut flag = false;

        if n == num_of_pins / 2 {
            buf.push_str("| GND\n");
            flag = true;
        }

        if n == num_of_pins {
            buf.push_str("| VCC\n\n");
            flag = true;
        }

        if gal_type == ::interop::GAL16V8 || gal_type == ::interop::GAL20V8 {
            if mode == ::interop::MODE3 && n == 1 {
                buf.push_str("| Clock\n");
                flag = true;
            }

            if mode == ::interop::MODE3 {
                if gal_type == ::interop::GAL16V8 && n == 11 {
                    buf.push_str("| /OE\n");
                    flag = true;
                }

                if gal_type == ::interop::GAL20V8 && n == 13 {
                    buf.push_str("| /OE\n");
                    flag = true;
                }
            }
        }

        if gal_type == ::interop::GAL22V10 && n == 1 {
            buf.push_str("| Clock/Input\n");
            flag = true;
        }

        // OLMC pin?
        // Second condition is a hack as VCC is a dummy OLMC on a 22V10.
        if is_olmc(gal_type, n) && n < 24 {
            let k = pin_to_olmc(gal_type, n);
            if olmc_pin_types[k] != INPUT {
                if olmc_pin_types[k] != 0 {
                    buf.push_str("| Output\n");
                } else {
                    buf.push_str("| NC\n");
                }
            } else {
                buf.push_str("| Input\n");
            }
        } else {
            if !flag {
                buf.push_str("| Input\n");
            }
        }
    }

    return buf;
}

fn make_row(buf: &mut String, num_of_col: usize, row: usize, data: &[u8]) {
    buf.push_str(&format!("\n{:>3} ", row));

    for col in 0..num_of_col {
        if col % 4 == 0 {
            buf.push_str(" ");
        }

        if data[row * num_of_col + col] != 0 {
            buf.push_str("-");
        } else {
            buf.push_str("x");
        }
    }
}

const OLMC_SIZE_22V10: [i32; 12] = [ 9, 11, 13, 15, 17, 17, 15, 13, 11, 9, 1, 1 ];

fn get_size(gal_type: i32, olmc: usize) -> i32
{
    match gal_type {
    ::interop::GAL16V8  => 8,
    ::interop::GAL20V8  => 8,
    ::interop::GAL22V10 => OLMC_SIZE_22V10[olmc],
    ::interop::GAL20RA10 => 8,
    _ => panic!("Nope")
    }
}

fn make_fuse(gal_type: i32, pin_names: &[&str], gal_fuse: &[u8], gal_xor: &[u8], gal_ac1: &[u8], gal_s1: &[u8]) -> String {
    let mut buf = String::new();

    let (mut pin, num_olmcs) = match gal_type {
        ::interop::GAL16V8   => (19, 8),
        ::interop::GAL20V8   => (22, 8),
        ::interop::GAL22V10  => (23, 10),
        ::interop::GAL20RA10 => (23, 10),
        _ => panic!("Nope"),
    };

    let row_len = match gal_type {
        ::interop::GAL16V8   => ::interop::ROW_LEN_ADR16,
        ::interop::GAL20V8   => ::interop::ROW_LEN_ADR20,
        ::interop::GAL22V10  => ::interop::ROW_LEN_ADR22V10,
        ::interop::GAL20RA10 => ::interop::ROW_LEN_ADR20RA10,
        _ => panic!("Nope"),
    };

    let mut row = 0;

    for olmc in 0..num_olmcs {
        if gal_type == ::interop::GAL22V10 && olmc == 0 {
            // AR when 22V10
            buf.push_str("\n\nAR");
            make_row(&mut buf, row_len, row, gal_fuse);
            row += 1;
        }

        let num_rows = get_size(gal_type, olmc);

        // Print pin
        buf.push_str(&format!("\n\nPin {:>2} = ", pin));

        buf.push_str(&format!("{}", pin_names[pin - 1]));

        make_spaces(&mut buf, 13 - pin_names[pin - 1].len());

        match gal_type {
            ::interop::GAL16V8 => {
                buf.push_str(&format!("XOR = {:>1}   AC1 = {:>1}", gal_xor[19 - pin], gal_ac1[19 - pin]));
            }
            ::interop::GAL20V8 => {
                buf.push_str(&format!("XOR = {:>1}   AC1 = {:>1}", gal_xor[22 - pin], gal_ac1[22 - pin]));
            }
            ::interop::GAL22V10 => {
                buf.push_str(&format!("S0 = {:>1}   S1 = {:>1}", gal_xor[23 - pin], gal_s1[23 - pin]));
            }
            ::interop::GAL20RA10 => {
                buf.push_str(&format!("S0 = {:>1}", gal_xor[23 - pin]));
            }
            _ => panic!("Nope"),
        };

        for n in 0..num_rows {
            // Print all fuses of an OLMC
            make_row(&mut buf, row_len, row, gal_fuse);
            row += 1;
        }


        if gal_type == ::interop::GAL22V10 && olmc == 9 {
            // SP when 22V10
            buf.push_str("\n\nSP");
            make_row(&mut buf, row_len, row, gal_fuse);
        }

        pin -= 1;
    }

    buf.push_str("\n\n");
    return buf;
}

pub fn write_files(file_name: &str,
               config: &::jedec_writer::Config,
               gal_type: i32,
               mode: i32,
               pin_names: &[&str],
               olmc_pin_types: &[i32],
               gal_fuses: &[u8],
               gal_xor: &[u8],
               gal_s1: &[u8],
               gal_sig: &[u8],
               gal_ac1: &[u8],
               gal_pt: &[u8],
               gal_syn: bool,
               gal_ac0: bool) {
    let base = PathBuf::from(file_name);

    {
        let buf = ::jedec_writer::make_jedec(gal_type, config, gal_fuses, gal_xor, gal_s1, gal_sig, gal_ac1, gal_pt, gal_syn, gal_ac0);
        let mut file = File::create(base.with_extension("jed").to_str().unwrap()).unwrap();
        file.write_all(buf.as_bytes());
    }

    if config.gen_fuse != 0 {
        let buf = make_fuse(gal_type, pin_names, gal_fuses, gal_xor, gal_ac1, gal_s1);
        let mut file = File::create(base.with_extension("fus").to_str().unwrap()).unwrap();
        file.write_all(buf.as_bytes());
    }

    if config.gen_pin != 0 {
        let buf = make_pin(gal_type, pin_names, mode, olmc_pin_types);
        let mut file = File::create(base.with_extension("pin").to_str().unwrap()).unwrap();
        file.write_all(buf.as_bytes());
    }

    if config.gen_chip != 0 {
        let buf = make_chip(gal_type, pin_names);
        let mut file = File::create(base.with_extension("chp").to_str().unwrap()).unwrap();
        file.write_all(buf.as_bytes());
    }
}
