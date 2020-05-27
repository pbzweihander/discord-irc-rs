pub fn normalize_irc_nickname(s: &str) -> String {
    s.replace("!", "ǃ") // U+0021 -> U+01C3
        .replace("@", "＠") // U+0040 -> U+FE6B
        .replace(" ", "_")
}
