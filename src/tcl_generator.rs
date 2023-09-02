use string_builder::Builder;

use crate::display_elements::{DisplayColor, DisplayFormat};

pub struct TclGenerator {
    signals: Vec<(String, Option<DisplayColor>, Option<DisplayFormat>)>,
    signal_prefix: String,
}

impl TclGenerator {
    pub fn new(signal_prefix: String) -> Self {
        Self {
            signals: vec![],
            signal_prefix,
        }
    }

    pub fn add_signal(
        &mut self,
        signal: String,
        color: Option<DisplayColor>,
        format: Option<DisplayFormat>,
    ) -> &mut Self {
        self.signals.push((signal, color, format));
        self
    }

    pub fn add_empty(&mut self) -> &mut Self {
        self.signals.push(("".to_owned(), None, None));
        self
    }

    pub fn generate(self) -> String {
        let mut builder = Builder::new(300);

        builder.append("gtkwave::nop\n");
        builder.append("gtkwave::/Edit/Set_Trace_Max_Hier 2\n");
        builder.append("gtkwave::/View/Show_Filled_High_Values 1\n");
        builder.append("gtkwave::/View/Show_Wave_Highlight 1\n");
        builder.append("gtkwave::/View/Show_Mouseover 1\n");

        for signal in self.signals {
            if signal.0 == "" {
                builder.append("gtkwave::/Edit/Insert_Blank\n");
            } else {
                builder.append("gtkwave::addSignalsFromList \"");
                builder.append(&self.signal_prefix[..]);
                builder.append(&signal.0[..]);
                builder.append("\"\n");

                builder.append("gtkwave::highlightSignalsFromList \"");
                builder.append(&self.signal_prefix[..]);
                builder.append(&signal.0[..]);
                builder.append("\"\n");

                if let Some(color) = signal.1 {
                    builder.append(format!("gtkwave::/Edit/Color_Format/{color}\n"));
                }

                if let Some(format) = signal.2 {
                    builder.append(format!("gtkwave::/Edit/Data_Format/{format}\n"));
                }

                builder.append("gtkwave::/Edit/UnHighlight_All\n");
            }
        }

        builder.append("gtkwave::/Time/Zoom/Zoom_Best_Fit\n");

        builder.string().unwrap()
    }
}
