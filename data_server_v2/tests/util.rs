use actix_web::rt::spawn;
use get_port::Ops;
use once_cell::sync::Lazy;
use rand::Rng;
use reqwest::Client;
use rush_data_server::{
    configuration::{get_app_env_key, get_configuration, mail::MailSettings},
    model::{account::CreateAccountDto, instance::CreateInstanceDto},
    startup::Application,
    telemetry::init_telemetry,
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::process::{Child, Command};
use std::{
    env,
    io::{self, BufRead, BufReader},
    process::Stdio,
};
use surrealdb::{engine::any::Any, Surreal};

pub struct TestApp {
    pub app_address: String,
    pub db: Surreal<Any>,
    pub smtp_client: Option<TestSmtpServerClient>,
}

impl TestApp {
    pub async fn post_account(&self, body: &CreateAccountDto) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/account", &self.app_address))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn post_instance(&self, body: &CreateInstanceDto) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/instance", &self.app_address))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }
}

pub async fn spawn_app(test_settings: TestSettings) -> io::Result<TestApp> {
    let TestSettings { spawn_smtp } = test_settings;
    env::set_var(get_app_env_key(), "test");
    Lazy::force(&TRACING);

    let (configuration, smtp_client) = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.application.port = 0;

        let smtp_client = if spawn_smtp {
            let (smtp_server_handle, smtp_port, http_port) = spawn_smtp_server(&c.mail);
            c.mail.smtp_port = Some(smtp_port);
            Some(TestSmtpServerClient::new(
                c.mail.clone(),
                smtp_server_handle,
                http_port,
            ))
        } else {
            None
        };

        (c, smtp_client)
    };

    let application = Application::build(configuration).await?;
    let db = application.get_db_ref().clone();
    let app_address = format!("http://127.0.0.1:{}", application.port());
    spawn(application.run_until_stopped());

    Ok(TestApp {
        app_address,
        db,
        smtp_client,
    })
}

fn local_host_to_ip(host: &str) -> &str {
    if host == "localhost" {
        "127.0.0.1"
    } else {
        host
    }
}

fn get_free_ports(host: &str) -> (u16, u16) {
    let mut rng = rand::thread_rng();
    let mut port1: u16 = rng.gen();
    while !get_port::tcp::TcpPort::is_port_available(host, port1) {
        port1 += 1;
    }
    let mut port2: u16 = port1 + 1;
    while !get_port::tcp::TcpPort::is_port_available(host, port2) {
        port2 += 1;
    }
    (port1, port2)
}

fn spawn_mail_server(host: &str, smtp_port: u16, http_port: u16) -> Child {
    Command::new("mailtutan") // TODO: attempt to load path to mailtutan from env variable for cicd purposes
        .args([
            "--ip",
            host,
            "--smtp-port",
            &smtp_port.to_string(),
            "--http-port",
            &http_port.to_string(),
        ])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn test smtp server: {e}"))
        .unwrap()
}

pub fn spawn_smtp_server(settings: &MailSettings) -> (Child, u16, u16) {
    let host = local_host_to_ip(&settings.smtp_host);
    let (smtp_port, http_port) = get_free_ports(host);

    let mut smtp_server_handle = spawn_mail_server(host, smtp_port, http_port);

    let reader = BufReader::new(smtp_server_handle.stdout.take().unwrap());
    let mut lines = reader.lines();

    // Should get two lines of output indicating complete startup.
    let _ = lines.next().unwrap();
    let _ = lines.next().unwrap();

    (smtp_server_handle, smtp_port, http_port)
}

static TRACING: Lazy<io::Result<()>> = Lazy::new(|| {
    init_telemetry()?;

    Ok(())
});

#[derive(Debug)]
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

pub struct TestSettings {
    pub spawn_smtp: bool,
}
