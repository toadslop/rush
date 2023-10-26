use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use secrecy::ExposeSecret;

use crate::configuration::{mail::MailSettings, Environment};

// TODO: need to handle smtps

const DEFAULT_SMTP_PORT: u16 = 25;

pub async fn init_mailer(
    settings: MailSettings,
    app_environment: &Environment,
) -> AsyncSmtpTransport<Tokio1Executor> {
    let connection = format!(
        "smtp://{}:{}",
        settings.smtp_host,
        settings.smtp_port.unwrap_or(DEFAULT_SMTP_PORT)
    );

    let mailer = if *app_environment == Environment::Prod {
        let credentials = Credentials::new(
            settings
                .smtp_username
                .expect("An smtp_username must be provided in production."),
            settings
                .smtp_password
                .expect("an smtp_password must be provided in production")
                .expose_secret()
                .to_owned(),
        );
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&connection)
            .expect("Failed to handle mail relay from settings")
            .credentials(credentials)
            .build()
    } else {
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::from_url(&connection)
            .expect("Failed to build smtp host");

        if let (Some(username), Some(password)) = (settings.smtp_username, settings.smtp_password) {
            builder = builder.credentials(Credentials::new(
                username,
                password.expose_secret().to_owned(),
            ));
        };

        builder.build()
    };

    // Some tests do not need an smtp server, so we don't test the connection
    // We test if the environment is explicitly not set to test to handle some
    // possible edge cases
    #[cfg(test)]
    if *app_environment != Environment::Test {
        mailer
            .test_connection()
            .await
            .expect("Failed to connect to SMTP server");
    }

    mailer
}
