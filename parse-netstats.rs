// Rust 0.12.0-dev
// Enable regex! macro.
#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

// Standard imports.
use std::io::BufferedReader;
use std::io::File;
use std::path::Path;

fn main() {
    // BufferedReader is used to read the log file.
    let path = Path::new("/var/log/netstats-history.log");
    let mut reader = BufferedReader::new(File::open(&path));

    println!("[i] Parsing filename: {}", path.as_str().unwrap());

    // I would like to implement the parsing by using a function that operates on BufferedReader,
    // because later I would like also to add support for gzipped log files. This way I hope I
    // won't be forced to change this function.
    parse_file(&mut reader);
}

fn parse_file<R: Reader>(reader: &mut BufferedReader<R>) {
    // Regular expression compiled at compilation time. It will be used to extract necessary
    // information from my log file.
    let date_re = regex!("^(.*?) @.*");
    let if_re   = regex!("([a-z0-9A-Z]+) RX ([0-9]+) TX ([0-9]+)");

    let mut lineno: int = 0;

    // I would like to lazily iterate on all lines. I don't want to read the whole log file
    // into memory, because I can process one line by one independently.
    for line in reader.lines() {
        // I would like to trim \n's here, so I'm using trim(). I can't use it on 'line', because
        // the type of 'line' doesn't implement trim() (it's IoResult<String>). So, the match
        // statement below unwraps the String from IoResult<String>.
        let linestr = match line {
            Ok(str) => str,
            Err(x)  => "".to_string()
        }; // or: let linestr = line.unwrap_or("".to_string());

        // Also, after unwrapping 'line', I'm getting the type of 'String', which doesn't implement
        // trim(), so I need to convert String to str.
        let lineslice = linestr.as_slice().trim();

        // Locate the date in the log line. This is done by using the `date_re` regular expression.
        let rfc2822_date_str = match date_re.captures(lineslice) {
            // `.captures(lineslice)` above returns Option<...> wrapping type. I need to unwrap the
            // enclosing type. I'm unwrapping it here, but only if Option<> contains something. If
            // it doesn't contain anything, it means that the regular expression didn't match
            // anything in the buffer. So:
            //
            // In case there are any matches:
            Some(caps) => caps.at(1), // Return 1st captured match.

            // In case of no matches:
            None       => fail!("Can't find the date in line {}", lineno)
        };

        println!("date: {}", rfc2822_date_str);

        // Here I'm trying to match the rest of the line against `if_re` regular expression. There
        // can be multiple interfaces defined, each with its own RX and TX values, so I need to use
        // a loop that will try to find multiple occurencies of `if_re` in the whole line.

        // If there are no matches, the loop simply doesn't iterate even once.
        for caps in if_re.captures_iter(lineslice) {
            let ifname = caps.at(1);
            let rxbytes = caps.at(2);
            let txbytes = caps.at(3);

            println!("if: {}, rx: {}, tx: {}", ifname, rxbytes, txbytes);
        }

        // Whole log is quite long, so for debug purposes I'm just limiting the parsing program
        // only to work with first 10 lines. Later this limitation will be removed.
        lineno += 1;
        if lineno > 10 { break; }
    }
}