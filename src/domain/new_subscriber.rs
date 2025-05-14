use super::{SubscriberEmail, SubscriberName};

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName
}

impl NewSubscriber {
    fn parse(name: String, email: String) -> Result<Self, String> {
        let name = SubscriberName::parse(name)?;
        let email = SubscriberEmail::parse(email)?;
        Ok(Self {
            name,
            email
        })
    }
}