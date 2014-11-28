extern crate getopts;
extern crate regex;

use getopts::{optopt,optflag,getopts,OptGroup};
use std::os;
use std::io::{BufferedReader,File};
use regex::Regex;

struct Interface<'a> {
    ifname: &'a str,
    bytes_rx: u64,
    bytes_tx: u64
}

struct RegularExpressions {
    date_re: Regex,
    if_re:   Regex
}

fn syntax(bin_name: &str) {
    println!("Syntax:");
    println!("");
    println!("    {} <options>", bin_name);
    println!("");
    println!("Options:");
    println!("");
    println!("    -i <infile>       log file name (from /var/log)");
    println!("    -h                this screen");
    println!("");
    println!("Output will be printed to stdout.");
}

fn syntax_error(bin_name: &str, f: getopts::Fail_) {
    println!("{}", f);
    println!("");
    syntax(bin_name);
}

fn main() {
    let args = os::args();
    let program = args[0].as_slice();
    let args = args.tail();

    let opts = [
        optopt("i", "in", "input filename", "NAME"),
        optflag("h", "help", "this screen")
    ];

    let matches = match getopts(args, &opts) {
        Ok(m) => m,
        Err(f) => { syntax_error(program, f); return }
    };

    if matches.opt_present("h") {
        syntax(program);
        return;
    }

    if ! matches.opt_present("i") {
        syntax(program);
        return;
    }

    let file_to_process = matches.opt_str("i").unwrap();
    println!("Processing file '{}'.", file_to_process.as_slice());

    if ! do_work(file_to_process.as_slice()) {
        println!("Processing failed.");
    }
}

fn do_work(filename: &str) -> bool {
    let file = match File::open(&Path::new(filename)) {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening input file: {}", filename);
            return false
        }
    };

    let res = &RegularExpressions {
        date_re: Regex::new("^(.*) @ ").unwrap(),
        if_re:   Regex::new("(.*?) RX ([0-9]+) TX ([0-9]+)[,]").unwrap()
    };

    let mut reader = BufferedReader::new(file);
    for line in reader.lines() {
        let line = match line {
            Ok(ref t) => t.trim(),
            Err(f) => break
        };

        let date: &str = parse_date(res, line);
        if date.len() == 0 {
            println!("Syntax error in log file: can't locate date.");
            return false;
        }

        let interface_data: Vec<Interface> = parse_interface_data(res, line.slice_from(date.len() + 3));

        if interface_data.len() < 1 {
            println!("Syntax error in log file: can't locate interface data.");
            return false;
        }

        println!("date: '{}'", date);
        for ifdata in interface_data.iter() {
            println!("if: {}, rx {}, tx {}", ifdata.ifname, ifdata.bytes_rx, ifdata.bytes_tx);
        }
    }

    return true;
}

fn parse_date<'a>(regex: &RegularExpressions, line: &str) -> &'a str {
    match regex.date_re.captures(line) {
        Some(ref caps) => caps.at(1).trim(),
        None => ""
    }
}

fn parse_interface_data<'a>(regex: &RegularExpressions, line: &str) -> Vec<Interface<'a>> {
    let mut vec: Vec<Interface> = Vec::new();
    for caps in regex.if_re.captures_iter(line) {
        vec.push(Interface {
            ifname: caps.at(1),
            bytes_rx: std::num::from_str_radix(caps.at(2), 10).unwrap_or(0),
            bytes_tx: std::num::from_str_radix(caps.at(3), 10).unwrap_or(0),
        });
    }
    vec
}
