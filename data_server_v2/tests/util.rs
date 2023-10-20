use actix_web::rt::spawn;
use get_port::Ops;
use lettre::transport::smtp;
use once_cell::sync::Lazy;
use reqwest::Client;
use rush_data_server::{
    configuration::{get_app_env_key, get_configuration, mail::MailSettings, Settings},
    database::init_db,
    mailer::init_mailer,
    telemetry::init_telemetry,
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::process::{Child, Command};
use std::{env, io, net::TcpListener};
use surrealdb::{engine::any::Any, Surreal};

pub async fn spawn_app() -> io::Result<(String, Surreal<Any>, TestSmtpServerClient)> {
    env::set_var(get_app_env_key(), "test");
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener
        .local_addr()
        .expect("Failed to get local TCP port from listener")
        .port();
    let Settings {
        database,
        mut mail,
        application,
    } = get_configuration().expect("Failed to read configuration.");
    let (smtp_server_handle, smtp_port, http_port) = spawn_smtp_server(&mail);
    mail.smtp_port = Some(smtp_port);
    let db = init_db(database).await.expect("Could not initialize db");
    let mailer = init_mailer(mail.clone(), application.environment).await;
    let server = rush_data_server::run(listener, db.clone(), mailer);
    spawn(server);

    Ok((
        format!("http://127.0.0.1:{}", port),
        db,
        TestSmtpServerClient::new(mail, smtp_server_handle, http_port),
    ))
}

pub fn spawn_smtp_server(settings: &MailSettings) -> (Child, u16, u16) {
    let smtp_host = if settings.smtp_host == "localhost" {
        "127.0.0.1"
    } else {
        &settings.smtp_host
    };
    let mut smtp_port: u16 = get_port::tcp::TcpPort::any(smtp_host)
        .expect("Failed to get a random TCP port for the test mail server");
    println!("smtp port {smtp_port}");

    while !get_port::tcp::TcpPort::is_port_available(smtp_host, smtp_port) {
        smtp_port += 1;
    }

    let http_host = settings
        .http_host
        .as_ref()
        .map(|host| {
            if host == "localhost" {
                "127.0.0.1"
            } else {
                host
            }
        })
        .unwrap_or("127.0.0.1");

    let mut http_port: u16 = get_port::tcp::TcpPort::any(http_host)
        .expect("Failed to get a random TCP port for the test mail server");

    while !get_port::tcp::TcpPort::is_port_available(http_host, http_port) || http_port == smtp_port
    {
        http_port += 1;
    }

    println!("http port: {http_port}");

    let smtp_server_handle = Command::new("mailtutan")
        .args([
            "--ip",
            "127.0.0.1",
            "--smtp-port",
            &smtp_port.to_string(),
            "--http-port",
            &http_port.to_string(),
        ])
        .spawn()
        .map_err(|e| format!("Failed to spawn test smtp server: {e}"))
        .unwrap();
    smtp_server_handle.stdout.as_ref().map(|f| {
        println!("{:?}", f);
        Some(f)
    });

    (smtp_server_handle, smtp_port, http_port)
}

static TRACING: Lazy<io::Result<()>> = Lazy::new(|| {
    init_telemetry()?;

    Ok(())
});

pub struct TestSmtpServerClient {
    settings: MailSettings,
    client: reqwest::Client,
    server_handle: Child,
    http_port: u16,
}

impl TestSmtpServerClient {
    pub fn new(settings: MailSettings, server_handle: Child, http_port: u16) -> Self {
        Client::new();
        Self {
            settings,
            client: Client::new(),
            server_handle,
            http_port,
        }
    }

    const MESSAGES_ENDPOINT: &str = "/api/messages";

    pub async fn get_messages(&self) -> Vec<MailtutanJsonMail> {
        let host = self
            .settings
            .http_host
            .as_ref()
            .expect("Test environment needs an http host for the smtp server but found None");

        let mut req = self.client.get(format!(
            "http://{host}:{}{}",
            self.http_port,
            Self::MESSAGES_ENDPOINT
        ));

        if let (Some(username), Some(password)) =
            (&self.settings.smtp_username, &self.settings.smtp_password)
        {
            req = req.basic_auth(username, Some(password.expose_secret().to_owned()));
        };

        let body = req
            .send()
            .await
            .map_err(|e| format!("Failed to retrieve messages from Mailtutan: {e}"))
            .unwrap()
            .json()
            .await
            .expect("Failed to convert the list of messages to a string");

        dbg!(&body);
        body
    }
}

impl Drop for TestSmtpServerClient {
    fn drop(&mut self) {
        self.server_handle
            .kill()
            .expect("Failed to kill the test smtp server");
    }
}

#[derive(Debug, Deserialize)]
pub struct MailtutanJsonMail {
    pub id: usize,
    pub sender: String,
    pub recipients: Vec<String>,
    pub subject: String,
    pub created_at: String,
    pub attachments: Vec<String>,
    pub formats: Vec<String>,
}
