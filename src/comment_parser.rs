use crate::display_elements::{DisplayColor, DisplayFormat};

pub enum Operation {
    UpdateContext(ContextUpdate),
    AddEmpty,
    AddSignal(String),
}

pub enum ContextUpdate {
    UpdateColor(Option<DisplayColor>),
    UpdateFormat(DisplayFormat),
    SetOmit(bool),
    Reset,
}

pub struct CommentParser {}

impl CommentParser {
    pub fn parse_comment(comment: &str) -> Vec<Operation> {
        comment
            .split(['\n', ','].as_ref())
            .map(|token| Self::parse_token(token))
            .filter(|operation| operation.is_some())
            .into_iter()
            .map(|operation| operation.unwrap())
            .collect()
    }

    fn parse_token(token: &str) -> Option<Operation> {
        Some(match token {
            "omit" => Operation::UpdateContext(ContextUpdate::SetOmit(true)),
            "empty" => Operation::AddEmpty,
            "reset" => Operation::UpdateContext(ContextUpdate::Reset),
            _ if token.starts_with("add signal ") => {
                Operation::AddSignal(token["add signal ".len()..].to_owned())
            }
            _ if token.starts_with("format ") => Operation::UpdateContext(
                ContextUpdate::UpdateFormat(Self::parse_format(&token["format ".len()..])),
            ),
            _ if token.starts_with("color ") => Operation::UpdateContext(
                ContextUpdate::UpdateColor(Self::parse_color(&token["color ".len()..])),
            ),
            _ => return None,
        })
    }

    fn parse_format(format: &str) -> DisplayFormat {
        match format {
            "hex" => DisplayFormat::Hex,
            "decimal" => DisplayFormat::Decimal,
            "signed decimal" => DisplayFormat::SignedDecimal,
            "binary" => DisplayFormat::Binary,
            _ => DisplayFormat::Decimal,
        }
    }

    fn parse_color(color: &str) -> Option<DisplayColor> {
        Some(match color {
            "normal" => DisplayColor::Normal,
            "red" => DisplayColor::Red,
            "orange" => DisplayColor::Orange,
            "yellow" => DisplayColor::Yellow,
            "green" => DisplayColor::Green,
            "blue" => DisplayColor::Blue,
            "indigo" => DisplayColor::Indigo,
            "violet" => DisplayColor::Violet,
            "cycle" => DisplayColor::Cycle,
            _ => return None,
        })
    }
}
