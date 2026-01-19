pub fn format_hex_preview(bytes: &[u8], max_bytes: usize) -> String {
    let preview_len = std::cmp::min(max_bytes, bytes.len());
    let hex: String = bytes[..preview_len]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");

    if bytes.len() > preview_len {
        format!("[{hex} ...]")
    } else {
        format!("[{hex}]")
    }
}
