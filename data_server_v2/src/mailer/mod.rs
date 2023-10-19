use lettre::{transport::smtp::authentication::Credentials, SmtpTransport};
use secrecy::ExposeSecret;

use crate::configuration::mail::MailSettings;

pub fn init_mailer(settings: MailSettings) -> SmtpTransport {
    let credentials = Credentials::new(
        settings.smtp_username,
        settings.smtp_password.expose_secret().to_owned(),
    );
    SmtpTransport::relay(&settings.relay)
        .expect("Failed to handle mail relay from settings")
        .credentials(credentials)
        .build()
}
