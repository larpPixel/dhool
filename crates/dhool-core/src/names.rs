pub fn sanitize_stem(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_file_stems() {
        assert_eq!(sanitize_stem("Inferno Test_01"), "Inferno_Test_01");
        assert_eq!(sanitize_stem("flame-final"), "flame-final");
    }
}
