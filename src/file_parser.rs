use vhdl_lang::{Source, syntax::{tokens::{TokenStream, Tokenizer, Kind, Comment}, Symbols}, Diagnostic, data::{Contents, ContentReader}};

use crate::display_elements::{DisplayColor, Entity, Architecture, Signal, DisplayOption, DisplayFormat};

#[derive(PartialEq, Eq, Clone)]
struct Context {
    color: Option<DisplayColor>,
    omit: bool
}

pub struct FileParser<'a> {
    diagnostics: Vec<Diagnostic>,
    stream: TokenStream<'a>,

    entities: Vec<Entity>,
    current_entity: usize
}

#[derive(Debug)]
pub enum ParseError {
    EntityNotPresent,
    ArchitectureNotFound,
    ParsingError(Diagnostic),
    EndOfFile,
}

impl From<Diagnostic> for ParseError {
    fn from(value: Diagnostic) -> Self {
        Self::ParsingError(value)
    }
}

impl From<&Context> for Vec<DisplayOption> {
    fn from(value: &Context) -> Self {
        let mut options = vec![];

        if value.omit {
            options.push(DisplayOption::Omit);
        }

        if let Some(color) = value.color {
            options.push(DisplayOption::Color(color));
        }

        options
    }
}

impl<'a> FileParser<'a> {
    pub fn new(source: &'a Source, contents: &'a Contents, symbols: &'a Symbols) -> Self {
        let mut diagnostics = vec![];
        let tokenizer = Tokenizer::new(symbols, source, ContentReader::new(contents));
        let stream = TokenStream::new(tokenizer, &mut diagnostics);

        Self {
            diagnostics,
            stream,
            entities: vec![],
            current_entity: 0,
        }
    }

    pub fn find_next_entity(&mut self) -> Result<Entity, ParseError> {
        if self.current_entity < self.entities.len() {
            self.current_entity += 1;
            return Ok(self.entities[self.current_entity - 1].clone());
        }

        if self.stream.skip_until(|k| k == Kind::Entity || k == Kind::Architecture).is_err() {
            return Err(ParseError::EndOfFile);
        }

        if self.stream.peek_kind().unwrap() == Kind::Entity {
            let entity = Self::parse_entity(&mut self.stream)?;
            self.entities.push(entity.clone());
            self.current_entity += 1;

            Ok(entity)
        } else {
            let architecture = Self::parse_architecture(&mut self.stream)?;

            if let Some(entity) = self.entities.iter_mut().find(|e| e.name() == architecture.entity_name()) {
                entity.add_architecture(architecture);
            }

            self.find_next_entity()
        }
    }

    pub fn parse_entity_architecture(&mut self, mut entity: Entity) -> Result<Entity, ParseError> {
        let Some(found_entity) = self.entities.iter_mut().find(|e| e.name() == entity.name()) else {
            return Err(ParseError::EntityNotPresent);
        };

        if entity.architecture().is_some() {
            return Ok(entity);
        }

        if found_entity.architecture().is_some() {
            return Ok(found_entity.clone());
        }

        if self.stream.skip_until(|k| k == Kind::Entity || k == Kind::Architecture).is_err() {
            return Err(ParseError::EndOfFile);
        }

        if self.stream.peek_kind().unwrap() == Kind::Entity {
            let entity = Self::parse_entity(&mut self.stream)?;
            self.entities.push(entity.clone());
            return self.parse_entity_architecture(entity)
        }

        let architecture = Self::parse_architecture(&mut self.stream)?;
        if architecture.entity_name() == entity.name() {
            entity.add_architecture(architecture.clone());
            found_entity.add_architecture(architecture);

            return Ok(entity);
        }

        if let Some(matched_entity) = self.entities.iter_mut().find(|e| e.name() == architecture.entity_name()) {
            matched_entity.add_architecture(architecture);
        }

        self.parse_entity_architecture(entity)
    }

    fn parse_architecture(stream: &mut TokenStream) -> Result<Architecture, ParseError> {
        stream.expect_kind(Kind::Architecture)?;

        let architecture_name = Self::parse_identifier(stream)?;

        stream.expect_kind(Kind::Of)?;

        let entity_name = Self::parse_identifier(stream)?;

        stream.expect_kind(Kind::Is)?;

        let mut context = Context { color: None, omit: false };
        let mut signals = vec![];

        while !stream.next_kind_is(Kind::Begin) {
            let token = stream.peek().ok_or(ParseError::EndOfFile)?;

            if let Some(comments) = &token.comments {
                for comment in &comments.leading {
                    Self::update_context(&mut context, comment);
                }
            }

            match token.kind {
                Kind::Signal => signals.append(Self::parse_signals(stream, &context)?.as_mut()),
                Kind::Begin => break,
                _ => stream.skip(),
            }
        }

        let architecture = Architecture::new(architecture_name, entity_name, signals);

        Ok(architecture)
    }

    fn parse_signals(stream: &mut TokenStream, context: &Context) -> Result<Vec<Signal>, ParseError> {
        stream.expect_kind(Kind::Signal)?;

        let mut signal_names = vec![];
        signal_names.push(Self::parse_identifier(stream)?);

        while stream.peek_kind().ok_or(ParseError::EndOfFile)? != Kind::Colon {
            stream.skip();
            signal_names.push(Self::parse_identifier(stream)?);
        }

        stream.skip();

        let signal_type = Self::parse_identifier(stream)?;

        stream.skip_until(|k| k == Kind::SemiColon)?;
        let semicolon_token = stream.peek().ok_or(ParseError::EndOfFile)?;

        let options: Vec<DisplayOption> = if let Some(comments) = &semicolon_token.comments {
            if let Some(trailing) = &comments.trailing {
                let mut context = context.clone();
                Self::update_context(&mut context, trailing);
                (&context).into()
            } else {
                context.into()
            }
        } else {
            context.into()
        };

        let mut signals = vec![];
        for signal_name in signal_names {
            let mut options = options.clone();
            if signal_type.starts_with("std_logic_vector") {
                options.push(DisplayOption::Format(DisplayFormat::Binary));
            }

            signals.push(Signal::new(signal_name, options));
        }

        Ok(signals)
    }

    fn parse_entity(stream: &mut TokenStream) -> Result<Entity, ParseError> {
        stream.expect_kind(Kind::Entity)?;

        let name = Self::parse_identifier(stream)?;
        Ok(Entity::new(name))
    }

    fn parse_identifier(stream: &mut TokenStream) -> Result<String, ParseError> {
        let token = stream.peek_expect()?;
        let identifier = token.to_identifier_value()?;

        stream.skip();

        Ok(identifier.item.name_utf8())
    }

    fn update_context(context: &mut Context, comment: &Comment) {
        let commands = comment.value.split(['\n', ','].as_ref());

        for command in commands {
            match command.trim() {
                "omit" => context.omit = true,
                "reset" => {
                    context.color = None;
                    context.omit = false;
                },
                _ if command.trim().starts_with("color ") => {
                    let color = command["color ".len()..].trim();

                    let color = match color {
                        "normal" => DisplayColor::Normal,
                        "red" => DisplayColor::Red,
                        "orange" => DisplayColor::Orange,
                        "yellow" => DisplayColor::Yellow,
                        "green" => DisplayColor::Green,
                        "blue" => DisplayColor::Blue,
                        "Indigo" => DisplayColor::Indigo,
                        "Violet" => DisplayColor::Violet,
                        "Cycle" => DisplayColor::Cycle,
                        _ => DisplayColor::Normal,
                    };

                    context.color = Some(color);
                },
                _ => ()
            }
        }
    }
}
