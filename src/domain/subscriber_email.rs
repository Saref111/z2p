use validator::ValidateEmail;

#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if !s.validate_email() {
            return Err(format!("{} is not a valid subscriber email.", s));
        };
        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        SubscriberEmail::parse(value)
    }
}

#[cfg(test)]
mod test {
    use crate::domain::SubscriberEmail;
    use claim::assert_err;
    use fake::{Fake, faker::internet::en::SafeEmail};
    use quickcheck::{Arbitrary, Gen};

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary(_g: &mut Gen) -> Self {
            let mut rng = rand::rng();
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[quickcheck_macros::quickcheck]
    fn full_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
