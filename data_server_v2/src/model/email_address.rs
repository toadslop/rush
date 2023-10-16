use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct EmailAddress(String);

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::model::email_address::{EmailAddress, MAX_EMAIL_LENGTH};
//     use claims::{assert_err, assert_ok};
//     use fake::{faker::internet::en::FreeEmailProvider, Fake};

//     #[test]
//     fn email_address_at_or_below_max_length_is_valid() {
//         let provider = FreeEmailProvider().fake::<String>();
//         let address = format!(
//             "{}@{}",
//             "ё".repeat(MAX_EMAIL_LENGTH - provider.len() - 1),
//             provider
//         );
//         assert_ok!(EmailAddress::try_from(address.as_str()));
//     }
//     #[test]
//     fn email_address_above_max_length_is_invalid() {
//         let provider = FreeEmailProvider().fake::<String>();
//         let address = format!(
//             "{}@{} ",
//             "a".repeat(MAX_EMAIL_LENGTH + provider.len()),
//             provider
//         );
//         assert_err!(EmailAddress::try_from(address.as_str()));
//     }
// }
