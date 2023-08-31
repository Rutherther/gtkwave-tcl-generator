pub mod file_parser;
pub mod display_elements;
pub mod tcl_generator;

use std::{path::PathBuf, fs::File, io::Write};

use clap::{arg, Parser};
use file_parser::FileParser;
use glob::glob;
use tcl_generator::TclGenerator;
use vhdl_lang::{syntax::Symbols, Source};

use crate::display_elements::{Signal, DisplayOption, DisplayColor};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'f', long = "folder")]
    folder: PathBuf,
    #[arg(short = 't', long = "testbench")]
    testbench: String,
    #[arg(short = 'o', long = "output")]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let find_wildcard = cli.folder.to_str().unwrap().to_owned() + "/**/*.vhd";
    println!("Going to look into {find_wildcard}");

    let matching_files = glob(&find_wildcard).unwrap();

    let symbols = Symbols::default();

    let mut found = false;
    for file_result in matching_files {
        let file = file_result.unwrap();

        let source = Source::from_latin1_file(file.as_path()).unwrap();
        let contents = source.contents();
        let mut parser = FileParser::new(&source, &contents, &symbols);

        match parser.find_next_entity() {
            Ok(entity) =>  {
                if entity.name() != &cli.testbench[..] {
                    continue;
                }
                found = true;

                println!("Found the testbench.");

                let entity = parser.parse_entity_architecture(entity).unwrap();
                let architecture = entity.architecture().unwrap();

                let mut generator = TclGenerator::new("top.".to_owned() + &cli.testbench + ".");
                generator.add_signal(&Signal::new("clk".to_owned(), vec![DisplayOption::Color(DisplayColor::Indigo)]))
                        .add_signal(&Signal::new("rst".to_owned(), vec![DisplayOption::Color(DisplayColor::Indigo)]))
                        .add_empty()
                        .zoom_out();

                for signal in architecture.signals() {
                    if signal.name() == "clk" || signal.name() == "rst" {
                        continue;
                    }

                    generator.add_signal(signal);
                }

                let generated = generator.generate();

                let mut file = File::create(&cli.output).unwrap();
                file.write_all(generated.as_bytes()).unwrap();

                break;
            },
            Err(err) => {
                println!("{:?}", err);
            }
        }

        if found {
            break;
        }
    }

    if !found {
        println!("Could not find the entity.")
    }
}
