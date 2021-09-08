#[derive(Debug, Clone)]
pub struct Label {
    label: String,
    display: Option<String>,
}

impl Label {
    pub fn new(label: String, display: Option<String>) -> Self {
        Self { label, display }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn display(&self) -> &str {
        self.display.as_ref().unwrap_or(&self.label)
    }

    pub fn from_label(label: String, max_label_length: usize) -> Self {
        let len = label.len();
        let display = if len > max_label_length {
            let len_trim = std::cmp::min(len - max_label_length, 3);

            let label = &label[0..(max_label_length - len_trim)];
            let padding = ".".repeat(len_trim);
            Some(format!("{}{}", label, padding))
        } else if len < max_label_length {
            let padding = " ".repeat(max_label_length - len);
            Some(format!("{}{}", label, padding))
        } else {
            None
        };

        Self::new(label, display)
    }
}
