use unicode_segmentation::UnicodeSegmentation;

pub fn left(width: usize, text: &str) -> String {
    let mut out = String::from(text);
    if width >= text.len() {
        out.push_str(&spaces(width - text.len()));
    }

    out
}

pub fn right(width: usize, text: &str) -> String {
    let mut out = String::new();
    if width >= text.len() {
        out.push_str(&spaces(width - text.len()));
    }
    out.push_str(text);

    out
}

#[test]
fn test_print_left_right() {
    assert_eq!(left(5, "AB"), "AB   ");
    assert_eq!(right(5, "AB"), "   AB");
}

pub fn top(height: usize, text: &str) -> Vec<String> {
    let mut out = text
        .graphemes(true)
        .map(String::from)
        .collect::<Vec<String>>();
    let len = out.len();
    if height > len {
        for _ in 0..(height - len) {
            out.push(String::from(" "));
        }
    }

    out
}

pub fn bottom(height: usize, text: &str) -> Vec<String> {
    let mut chars = text
        .graphemes(true)
        .map(String::from)
        .collect::<Vec<String>>();
    let len = chars.len();
    let mut out: Vec<String> = Vec::new();
    if height > len {
        for _ in 0..(height - len) {
            out.push(String::from(" "));
        }
    }
    out.append(&mut chars);

    out
}

#[test]
fn test_print_top_bottom() {
    assert_eq!(top(5, "AB"), vec!["A", "B", " ", " ", " "]);
    assert_eq!(bottom(5, "AB"), vec![" ", " ", " ", "A", "B"]);
}

pub fn spaces(width: usize) -> String {
    chars(width, ' ')
}

pub fn chars(width: usize, c: char) -> String {
    let mut out = String::new();
    for _ in 0..width {
        out.push(c);
    }

    out
}
