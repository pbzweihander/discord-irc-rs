use unicode_segmentation::UnicodeSegmentation;

pub fn normalize_irc_nickname(s: &str) -> String {
    s.replace('!', "ǃ") // U+0021 -> U+01C3
        .replace('@', "＠") // U+0040 -> U+FE6B
        .replace(' ', "_")
}

pub fn insert_zero_width_spaces_into_nickname(nick: &str) -> String {
    let graphemes: Vec<_> = nick.grapheme_indices(true).map(|entry| entry.0).collect();
    match graphemes.len() {
        0 => String::new(),
        1 => nick.to_string(),
        2 => {
            let mut s = String::with_capacity(nick.len() + "\u{200B}".len());

            let a = graphemes[1];

            s.push_str(&nick[..a]);
            s.push('\u{200B}');
            s.push_str(&nick[a..]);
            s
        }
        count => {
            let mut s = String::with_capacity(nick.len() + "\u{200B}".len() * 2);

            let a = graphemes[count / 3];
            let b = graphemes[count * 2 / 3];

            s.push_str(&nick[..a]);
            s.push('\u{200B}');
            s.push_str(&nick[a..b]);
            s.push('\u{200B}');
            s.push_str(&nick[b..]);
            s
        }
    }
}

#[test]
pub fn test_insert_boms_into_nickname() {
    let f = insert_zero_width_spaces_into_nickname;

    assert_eq!(f("기이다란닉네임"), "기이\u{200B}다란\u{200B}닉네임");
    assert_eq!(f("기다란닉네임"), "기다\u{200B}란닉\u{200B}네임");
    assert_eq!(f("가나다라마"), "가\u{200B}나다\u{200B}라마");
    assert_eq!(f("지현지현"), "지\u{200B}현\u{200B}지현");
    assert_eq!(f("김지현"), "김\u{200B}지\u{200B}현");
    assert_eq!(f("지현"), "지\u{200B}현");
    assert_eq!(f("젼"), "젼");
    assert_eq!(f(""), "");

    assert_eq!(f("a̐éö̲"), "a̐\u{200B}é\u{200B}ö̲");
}
