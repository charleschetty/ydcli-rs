pub mod color;

pub fn trim_str(t: &str) -> Option<String> {
    let t = t.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_owned())
    }
}

pub fn format_data(value: &str, color: &str) -> String {
    format!("{color}{value}{blank}", value = value,blank= "\x1b[0m")
}
