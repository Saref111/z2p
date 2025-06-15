use rand::{Rng, distr::Alphanumeric};
use serde::de;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct ConfirmationToken(String);

impl ConfirmationToken {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let token = std::iter::repeat_with(|| rng.sample(Alphanumeric))
            .map(char::from)
            .take(25)
            .collect();
        Self(token)
    }

    pub fn parse(s: String) -> Result<Self, String> {
        let trim_or_whitespace = s.trim().is_empty();
        let is_wrong_length = s.graphemes(true).count() != 25;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|c| forbidden_characters.contains(&c));

        if trim_or_whitespace || is_wrong_length || contains_forbidden_chars {
            Err(format!("{} is invalid confirmation token.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl Default for ConfirmationToken {
    fn default() -> Self {
        Self::new()
    }
}

struct ConfirmationTokenVisitor;

impl de::Visitor<'_> for ConfirmationTokenVisitor {
    type Value = ConfirmationToken;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid confirmation token string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ConfirmationToken::parse(value.to_string()).map_err(de::Error::custom)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ConfirmationToken::parse(value).map_err(de::Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for ConfirmationToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(ConfirmationTokenVisitor)
    }
}

impl AsRef<str> for ConfirmationToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::domain::ConfirmationToken;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_25_grapheme_long_name_is_valid() {
        let token = "a".repeat(25);
        assert_ok!(ConfirmationToken::parse(token));
    }
    #[test]
    fn a_token_longer_than_25_graphemes_is_rejected() {
        let token = "a".repeat(26);
        assert_err!(ConfirmationToken::parse(token));
    }
    #[test]
    fn a_token_shorter_than_25_graphemes_is_rejected() {
        let token = "a".repeat(24);
        assert_err!(ConfirmationToken::parse(token));
    }
    #[test]
    fn whitespace_only_tokens_are_rejected() {
        let token = " ".to_string();
        assert_err!(ConfirmationToken::parse(token));
    }

    #[test]
    fn empty_string_is_rejected() {
        let token = "".to_string();
        assert_err!(ConfirmationToken::parse(token));
    }
    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for token in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let token = token.to_string().repeat(25);
            assert_err!(ConfirmationToken::parse(token));
        }
    }
    #[test]
    fn a_valid_token_is_parsed_successfully() {
        let token = ConfirmationToken::new();
        let token = token.as_ref().to_string();
        assert_ok!(ConfirmationToken::parse(token));
    }

    #[test]
    fn a_valid_token_can_be_deserialized() {
        let token = ConfirmationToken::new();
        let token_json_str = format!("\"{}\"", token.as_ref());
        let de_token: ConfirmationToken = serde_json::from_str(&token_json_str).unwrap();

        assert_eq!(token.as_ref(), de_token.as_ref())
    }

    #[test]
    fn deserialization_is_rejected_on_invalid_token() {
        let token_json_str = format!("\"{}\"", "some { invalid ] [ token string");
        let result: serde_json::Result<ConfirmationToken> = serde_json::from_str(&token_json_str);
        assert_err!(result);
    }
}
