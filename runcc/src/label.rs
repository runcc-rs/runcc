pub fn display_label(label: &str, max_label_length: usize) -> String {
    let len = label.len();
    if len > max_label_length {
        let len_trim = std::cmp::min(len - max_label_length, 3);

        let label = &label[0..(max_label_length - len_trim)];
        let padding = ".".repeat(len_trim);
        format!("{}{}", label, padding)
    } else if len < max_label_length {
        let padding = " ".repeat(max_label_length - len);
        format!("{}{}", label, padding)
    } else {
        label.to_string()
    }
}
