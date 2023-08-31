use std::{slice::Iter, fmt::Display};

#[derive(Eq, PartialEq, Clone)]
pub enum DisplayElement {
    Signal(Signal),
    Empty(Vec<DisplayOption>)
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum DisplayOption {
    Color(DisplayColor),
    Format(DisplayFormat),
    Omit
}

#[derive(Eq, PartialEq, Clone)]
pub struct Signal {
    name: String,
    options: Vec<DisplayOption>
}

impl From<&str> for Signal {
    fn from(value: &str) -> Self {
        Self {
            name: value.to_owned(),
            options: vec![]
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Entity {
    name: String,
    architecture: Option<Architecture>,
}

#[derive(Eq, PartialEq, Clone)]
pub struct Architecture {
    name: String,
    entity_name: String,
    signals: Vec<Signal>
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum DisplayColor {
    Normal,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
    Cycle,
}

impl Display for DisplayColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DisplayColor::Normal => "Normal",
            DisplayColor::Red => "Red",
            DisplayColor::Orange => "Orange",
            DisplayColor::Yellow => "Yellow",
            DisplayColor::Green => "Green",
            DisplayColor::Blue => "Blue",
            DisplayColor::Indigo => "Indigo",
            DisplayColor::Violet => "Violet",
            DisplayColor::Cycle => "Cycle",
        })
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum DisplayFormat {
    Hex,
    Decimal,
    SignedDecimal,
    Binary,
}

impl Display for DisplayFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DisplayFormat::Hex => "Hex",
            DisplayFormat::Decimal => "Decimal",
            DisplayFormat::SignedDecimal => "SignedDecimal",
            DisplayFormat::Binary => "Binary",
        })
    }
}

impl Signal {
    pub fn new(name: String, options: Vec<DisplayOption>) -> Self {
        Self {
            name,
            options
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn options(&self) -> Iter<'_, DisplayOption> {
        self.options.iter()
    }
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            architecture: None
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_architecture(&mut self, architecture: Architecture) {
        self.architecture = Some(architecture);
    }

    pub fn architecture(&self) -> Option<&Architecture> {
        self.architecture.as_ref()
    }
}

impl Architecture {
    pub fn new(name: String, entity_name: String, signals: Vec<Signal>) -> Self {
        Self {
            name,
            entity_name,
            signals,
        }
    }

    pub fn signals(&self) -> Iter<'_, Signal> {
        self.signals.iter()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }
}
