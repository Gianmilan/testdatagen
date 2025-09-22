use clap::{Arg, ArgMatches, Command};
use std::error::Error;
use std::fs::File;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let matches: ArgMatches = Command::new("testdatagen")
        .version("0.0.1")
        .author("GS")
        .about("Read CSV files, and generate entries") // Other formats coming in the future (hopefully)
        .arg(
            Arg::new("FILE")
                .help("Sets the input CSV path to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let filename = matches.get_one::<String>("FILE").expect("required");
    read_csv(filename)
}

fn read_csv<P: AsRef<Path>>(filename: P) -> Result<(), Box<dyn Error>> {
    let file = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.records() {
        let record = result?;
        println!("{:?}", record);
    }
    Ok(())
}
