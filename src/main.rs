extern crate getopts;
extern crate regex;
extern crate time;

use getopts::{optopt,optflag,getopts,OptGroup};
use std::os;
use std::io::{BufferedReader,File};
use std::collections::TreeMap;
use regex::Regex;

static ZuluTime: &'static str = "%a, %d %b %Y %T %z";

struct StringPool {
    pool: Vec<String>
}

struct DateEntry<'a> {
    dateobj: time::Tm,
    ifdata:  Vec<Interface>,
}

struct Interface {
    ifname: String,
    bytes_rx: u64,
    bytes_tx: u64,
}

struct InterfaceDelta {
    fname: String,
    delta_rx: u64,
    delta_tx: u64,
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
    let mut db: Vec<Vec<Interface>> = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(ref t) => t.trim(),
            Err(f) => break
        };

        let datestr: &str = parse_date(res, line);
        if datestr.len() == 0 {
            println!("Syntax error in log file: can't locate date.");
            return false;
        }

        let dateobj = match time::strptime(datestr, ZuluTime) {
            Ok(datetm) => datetm,
            Err(f) => {
                println!("Error parsing date: '{}'", datestr);
                println!("Error description: {}", f);
                return false;
            }
        };

        let interface_data: Vec<Interface> = parse_interface_data(res, line.slice_from(datestr.len() + 3), &dateobj);

        if interface_data.len() < 1 {
            println!("Syntax error in log file: can't locate interface data.");
            return false;
        }

        db.push(interface_data);
    }

    let ifnames = get_interface_names(&db);
    let mut ifmaps: TreeMap<&String, Vec<Interface>> = TreeMap::new();
    for ifname in ifnames.iter() {
        let mut ifvec = build_vec_for_interface(*ifname, &db);
        process_ifvec_table(&mut ifvec);

        println!("found {} entries for ifname {}", ifvec.len(), ifname);
        dump_ifvec(&ifvec);
        ifmaps.insert(*ifname, ifvec);
    }

    return true;
}

fn process_ifvec_table(ifvec: &mut Vec<Interface>) {
    for i in range(1, ifvec.len()) {
        let current_rx = ifvec[i].bytes_rx;
        let current_tx = ifvec[i].bytes_tx;

        let mut previous = &mut ifvec[i - 1];
        previous.bytes_rx = current_rx - previous.bytes_rx;
        previous.bytes_tx = current_tx - previous.bytes_tx;
    }

    ifvec.pop();
}

fn dump_ifvec(table: &Vec<Interface>) {
    for item in table.iter() {
        println!("ifname {:8} RX {:16} TX {:16}", item.ifname, item.bytes_rx, item.bytes_tx);
    }
}

fn build_vec_for_interface<'a>(ifname: &String, db: &'a Vec<Vec<Interface>>) -> Vec<Interface> {
    let mut out: Vec<Interface> = Vec::new();

    for entry in db.iter() {
        for ifentry in entry.iter() {
            if ifentry.ifname == *ifname {
                out.push(Interface { ifname: ifentry.ifname.to_string(), bytes_rx: ifentry.bytes_rx, bytes_tx: ifentry.bytes_tx });
            }
        }
    }

    return out;
}

fn get_interface_names<'a>(db: &'a Vec<Vec<Interface>>) -> Vec<&'a String> {
    let mut ifnames: Vec<&'a String> = Vec::new();
    for ifdata in db[0].iter() {
        ifnames.push(&ifdata.ifname);
    }

    ifnames
}

fn parse_date<'a>(regex: &RegularExpressions, line: &str) -> &'a str {
    match regex.date_re.captures(line) {
        Some(ref caps) => caps.at(1).trim(),
        None => ""
    }
}

fn parse_interface_data(regex: &RegularExpressions, line: &str, tm: &time::Tm) -> Vec<Interface> {
    let mut vec: Vec<Interface> = Vec::new();
    for caps in regex.if_re.captures_iter(line) {
        let string: &str = caps.at(1);
        let ifobj = Interface {
            ifname: string.to_string(),
            bytes_rx: std::num::from_str_radix(caps.at(2), 10).unwrap_or(0),
            bytes_tx: std::num::from_str_radix(caps.at(3), 10).unwrap_or(0),
        };

        vec.push(ifobj);
    }
    vec
}
