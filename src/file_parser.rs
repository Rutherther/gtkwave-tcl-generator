use vhdl_lang::{
    data::{ContentReader, Contents},
    syntax::{
        tokens::{Kind, TokenStream, Tokenizer},
        Symbols,
    },
    Diagnostic, Source,
};

#[derive(Eq, PartialEq)]
pub struct ParsedEntity {
    name: String,
    architecture: Option<ParsedArchitecture>,
}

impl ParsedEntity {
    pub fn name(&self) -> &str {
        &self.name[..]
    }

    pub fn architecture(&self) -> Option<&ParsedArchitecture> {
        self.architecture.as_ref()
    }
}

#[derive(Eq, PartialEq)]
pub struct ParsedArchitecture {
    name: String,
    entity_name: String,
    parts: Vec<ParsedArchitecturePart>,
}

impl ParsedArchitecture {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }

    pub fn parts(&self) -> &Vec<ParsedArchitecturePart> {
        &self.parts
    }
}

#[derive(PartialEq, Eq)]
pub struct ParsedSignal {
    name: String,
    signal_type: String,
    comment: Option<String>,
}

impl ParsedSignal {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn signal_type(&self) -> &str {
        &self.signal_type
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

#[derive(PartialEq, Eq)]
pub enum ParsedArchitecturePart {
    Signal(ParsedSignal),
    Comment(String),
}

pub struct FileParser {}

#[derive(Debug)]
pub enum ParseError {
    ArchitectureWithoutEntity,
    ParsingError(Diagnostic),
    EndOfFile,
}

impl From<Diagnostic> for ParseError {
    fn from(value: Diagnostic) -> Self {
        Self::ParsingError(value)
    }
}

impl FileParser {
    pub fn parse_file(
        source: &Source,
        contents: &Contents,
        symbols: &Symbols,
    ) -> Result<Vec<ParsedEntity>, ParseError> {
        let mut entities = vec![];
        let mut diagnostics = vec![];
        let tokenizer = Tokenizer::new(symbols, source, ContentReader::new(contents));
        let mut stream = TokenStream::new(tokenizer, &mut diagnostics);

        while stream
            .skip_until(|k| k == Kind::Entity || k == Kind::Architecture)
            .is_ok()
        {
            let kind = stream.peek_kind().unwrap();

            match kind {
                Kind::Entity => {
                    entities.push(Self::parse_entity(&mut stream)?);
                }
                Kind::Architecture => {
                    let architecture = Self::parse_architecture(&mut stream)?;
                    let entity = entities
                        .iter_mut()
                        .find(|e| e.name == architecture.entity_name)
                        .ok_or(ParseError::ArchitectureWithoutEntity)?;
                    entity.architecture = Some(architecture);
                }
                _ => panic!("Wrong kind. Skip until bugged."),
            }
        }

        Ok(entities)
    }

    fn parse_architecture(stream: &mut TokenStream) -> Result<ParsedArchitecture, ParseError> {
        stream.expect_kind(Kind::Architecture)?;

        let architecture_name = Self::parse_identifier(stream)?;

        stream.expect_kind(Kind::Of)?;

        let entity_name = Self::parse_identifier(stream)?;

        stream.expect_kind(Kind::Is)?;

        let mut parts: Vec<ParsedArchitecturePart> = vec![];

        loop {
            let token = stream.peek().ok_or(ParseError::EndOfFile)?;

            if let Some(comments) = &token.comments {
                for comment in &comments.leading {
                    parts.push(ParsedArchitecturePart::Comment(comment.value.clone()));
                }
            }

            match token.kind {
                Kind::Signal => parts.extend(
                    Self::parse_signals(stream)?
                        .into_iter()
                        .map(|s| ParsedArchitecturePart::Signal(s)),
                ),
                Kind::Begin => break,
                _ => stream.skip(),
            }
        }

        stream.skip_until(|k| k == Kind::Architecture)?;
        stream.skip_until(|k| k == Kind::SemiColon)?;

        let architecture = ParsedArchitecture {
            name: architecture_name,
            entity_name,
            parts,
        };

        Ok(architecture)
    }

    fn parse_signals(stream: &mut TokenStream) -> Result<Vec<ParsedSignal>, ParseError> {
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

        let comment = semicolon_token.comments.as_ref().and_then(|comments| {
            comments
                .trailing
                .as_ref()
                .map(|trailing| trailing.value.clone())
        });

        let mut signals = vec![];
        for signal_name in signal_names {
            signals.push(ParsedSignal {
                name: signal_name,
                signal_type: signal_type.clone(),
                comment: comment.clone(),
            });
        }

        Ok(signals)
    }

    fn parse_entity(stream: &mut TokenStream) -> Result<ParsedEntity, ParseError> {
        stream.expect_kind(Kind::Entity)?;

        let name = Self::parse_identifier(stream)?;

        stream.skip_until(|k| k == Kind::Entity)?;
        stream.skip_until(|k| k == Kind::SemiColon)?;

        Ok(ParsedEntity {
            name,
            architecture: None,
        })
    }

    fn parse_identifier(stream: &mut TokenStream) -> Result<String, ParseError> {
        let token = stream.peek_expect()?;
        let identifier = token.to_identifier_value()?;

        stream.skip();

        Ok(identifier.item.name_utf8())
    }
}
