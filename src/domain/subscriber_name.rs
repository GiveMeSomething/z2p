use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    // parse is the only way to build an instance of SubscriberName outside of domain module
    // Therefore, any instance of SubscriberName will satisfy our constrains
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty = s.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_chars = ['/', '(', ')', '<', '>', '[', ']', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty || is_too_long || contains_forbidden_chars {
            // panic!("{} is not a valid subscriber name", s);
            return Err(format!("{} is not a valid subscriber name", s));
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use crate::domain::subscriber_name::SubscriberName;

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_string_is_rejected() {
        let name = "    ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn a_256_grapheme_name_is_valid() {
        let name = "á".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_257_grapheme_name_is_rejected() {
        let name = "ớ".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn name_with_invalid_chars_is_rejected() {
        for name in &['/', '(', ')', '<', '>', '[', ']', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn valid_name_parse_successfully() {
        let name = "Minh Hoang Tien".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
