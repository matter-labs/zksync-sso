pub fn strip_0x(hex_string: &str) -> String {
    if hex_string.starts_with("0x") {
        hex_string.split("0x").collect::<Vec<_>>()[1].to_string()
    } else {
        hex_string.to_string()
    }
}
