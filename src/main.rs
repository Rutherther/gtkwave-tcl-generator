pub mod comment_parser;
pub mod display_elements;
pub mod file_parser;
pub mod tcl_generator;

use std::{fs::File, io::Write, path::PathBuf};

use clap::{arg, Parser};
use comment_parser::{CommentParser, ContextUpdate, Operation};
use display_elements::DisplayFormat;
use file_parser::{FileParser, ParsedArchitecturePart, ParsedEntity};
use glob::glob;
use tcl_generator::TclGenerator;
use vhdl_lang::{syntax::Symbols, Source};

use crate::display_elements::DisplayColor;

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

    let matching_files = glob(&find_wildcard).unwrap();

    let symbols = Symbols::default();

    let mut found = false;
    for file_result in matching_files {
        let file = file_result.unwrap();

        let source = Source::from_latin1_file(file.as_path()).unwrap();
        let contents = source.contents();
        let entities = FileParser::parse_file(&source, &contents, &symbols).unwrap();

        for entity in entities {
            if entity.name() != cli.testbench {
                continue;
            }
            found = true;
            println!("Found the testbench.");

            let tcl = generate_tcl(entity);

            let mut file = File::create(cli.output.clone()).unwrap();
            file.write_all(tcl.as_bytes()).unwrap();

            println!("Generated {}.", cli.output.display());

            break;
        }

        if found {
            break;
        }
    }

    if !found {
        println!("Could not find the entity.")
    }
}

#[derive(Eq, PartialEq, Clone)]
struct Context {
    color: Option<DisplayColor>,
    format: Option<DisplayFormat>,
    omit: bool,
}

impl Context {
    pub fn update<'a>(&mut self, updates: impl Iterator<Item = &'a ContextUpdate>) {
        for update in updates {
            match update {
                ContextUpdate::Reset => {
                    self.color = None;
                    self.format = None;
                    self.omit = false;
                }
                ContextUpdate::SetOmit(omit) => {
                    self.omit = omit.clone();
                }
                ContextUpdate::UpdateColor(color) => {
                    self.color = color.clone();
                }
                ContextUpdate::UpdateFormat(format) => {
                    self.format = Some(format.clone());
                }
            }
        }
    }

    pub fn fork<'a>(&self, updates: impl Iterator<Item = &'a ContextUpdate>) -> Context {
        let mut clone = self.clone();
        clone.update(updates);

        clone
    }

    pub fn decompose(&self) -> (Option<DisplayColor>, Option<DisplayFormat>, bool) {
        (self.color, self.format, self.omit)
    }
}

fn generate_tcl(entity: ParsedEntity) -> String {
    let architecture = entity.architecture().unwrap();

    let mut generator = TclGenerator::new("top.".to_owned() + entity.name() + ".");

    let mut context = Context {
        color: None,
        format: None,
        omit: false,
    };

    for part in architecture.parts() {
        match part {
            ParsedArchitecturePart::Comment(comment) => {
                let operations = CommentParser::parse_comment(&comment[..]);
                if let Some(operation) = operations.first() {
                    if let Operation::AddSignal(signal) = operation {
                        let context_operations = operations.iter().skip(1);
                        add_signal(
                            &mut generator,
                            signal.clone(),
                            None,
                            Some(context_operations),
                            &context,
                            true
                        );
                    } else {
                        let updates = operations.iter().filter_map(|op| match op {
                            Operation::UpdateContext(update) => Some(update),
                            Operation::AddEmpty => {
                                generator.add_empty();
                                None
                            }
                            _ => panic!(),
                        });
                        context.update(updates);
                    }
                }
            },
            ParsedArchitecturePart::Signal(signal) => {
                let mut operations = None;

                if let Some(comment) = signal.comment() {
                    operations = Some(CommentParser::parse_comment(comment));
                }

                add_signal(
                    &mut generator,
                    signal.name().to_owned(),
                    Some(signal.signal_type()),
                    operations.as_ref().map(|x| x.iter()),
                    &context,
                    false
                );
            }
        }
    }

    generator.generate()
}

fn add_signal<'a>(
    generator: &mut TclGenerator,
    signal: String,
    signal_type: Option<&str>,
    operations: Option<impl Iterator<Item = &'a Operation>>,
    context: &Context,
    override_omit: bool
) {
    let (color, mut format, omit) = if let Some(operations) = operations {
        let updates = operations.into_iter().filter_map(|op| {
            if let Operation::UpdateContext(update) = op {
                Some(update)
            } else {
                None
            }
        });

        context.fork(updates).decompose()
    } else {
        context.decompose()
    };

    if format.is_none() && signal_type.is_some() && signal_type.unwrap() == "std_logic_vector" {
        format = Some(DisplayFormat::Binary);
    }

    if override_omit || !omit {
        generator.add_signal(signal, color, format);
    }
}
