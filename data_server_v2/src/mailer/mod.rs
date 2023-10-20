use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use secrecy::ExposeSecret;

use crate::configuration::{mail::MailSettings, Environment};

pub async fn init_mailer(
    settings: MailSettings,
    app_environment: Environment,
) -> AsyncSmtpTransport<Tokio1Executor> {
    let connection = format!(
        "smtp://{}:{}",
        settings.smtp_host,
        settings.smtp_port.expect("An smpt_port is required")
    );
    println!("connection: {connection}");
    let mailer = if app_environment == Environment::Prod {
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
        println!("is test env");
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

    mailer
        .test_connection()
        .await
        .expect("Failed to connect to SMTP server");

    mailer
}
