use egzreader::EgzReader;
use std::env;
use std::fs::File;
use std::io;
use std::io::{stdout, BufReader, BufWriter};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("USAGE:");
        println!("$ auto-zcat FILE [FILE]...");
        println!("Either gzip or non-gzip files can be accepted.");
    }

    let w = stdout();
    let mut w = BufWriter::new(w.lock());

    args[1..]
        .iter()
        .filter_map(|a| File::open(a).ok())
        .map(|f| BufReader::new(EgzReader::new(f)))
        .for_each(|mut r| {
            io::copy(&mut r, &mut w).unwrap();
        });
}
