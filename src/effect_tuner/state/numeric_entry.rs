#[derive(Default, Clone)]
struct NumericEntryBuffer {
    buffer: String,
}

impl NumericEntryBuffer {
    fn displayed_text(&self) -> Option<&str> {
        (!self.buffer.is_empty()).then_some(self.buffer.as_str())
    }

    fn push(&mut self, character: char) -> bool {
        match character {
            '0'..='9' => {
                self.buffer.push(character);
                true
            }
            '.' | ',' => {
                if self.buffer.contains('.') {
                    return false;
                }
                if self.buffer.is_empty() {
                    self.buffer.push('0');
                } else if matches!(self.buffer.as_str(), "-" | "+") {
                    self.buffer.push('0');
                }
                self.buffer.push('.');
                true
            }
            '-' | '+' => {
                if self.buffer.is_empty() {
                    self.buffer.push(character);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn backspace(&mut self) -> bool {
        self.buffer.pop().is_some()
    }

    fn parsed_value(&self) -> Option<f32> {
        match self.buffer.as_str() {
            "" | "-" | "+" | "." | "-." | "+." => None,
            value => value.parse::<f32>().ok(),
        }
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }
}
