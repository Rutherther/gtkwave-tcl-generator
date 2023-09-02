use std::fmt::Display;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum DisplayOption {
    Color(DisplayColor),
    Format(DisplayFormat),
    Omit,
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
        write!(
            f,
            "{}",
            match self {
                DisplayColor::Normal => "Normal",
                DisplayColor::Red => "Red",
                DisplayColor::Orange => "Orange",
                DisplayColor::Yellow => "Yellow",
                DisplayColor::Green => "Green",
                DisplayColor::Blue => "Blue",
                DisplayColor::Indigo => "Indigo",
                DisplayColor::Violet => "Violet",
                DisplayColor::Cycle => "Cycle",
            }
        )
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
        write!(
            f,
            "{}",
            match self {
                DisplayFormat::Hex => "Hex",
                DisplayFormat::Decimal => "Decimal",
                DisplayFormat::SignedDecimal => "SignedDecimal",
                DisplayFormat::Binary => "Binary",
            }
        )
    }
}
