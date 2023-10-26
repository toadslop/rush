use actix_web::dev::Server;
use std::net::TcpListener;
use surrealdb::{engine::any::Any, Surreal};

use crate::{
    configuration::{app::ApplicationSettings, Settings},
    database::init::init_db,
    mailer::init_mailer,
    run,
};

pub struct Application {
    settings: ApplicationSettings,
    server: Server,
    db: Surreal<Any>,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let mut settings = configuration.application;
        let address = format!("{}:{}", &settings.host, &settings.port);

        let db = init_db(configuration.database, &settings.environment)
            .await
            .expect("Could not initialize db");
        let mailer = init_mailer(configuration.mail, &settings.environment).await;

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        settings.port = port;
        let server = run(listener, db.clone(), mailer, settings.clone()).await?;

        Ok(Self {
            settings,
            server,
            db,
        })
    }

    pub fn port(&self) -> u16 {
        self.settings.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn get_db_ref(&self) -> &Surreal<Any> {
        &self.db
    }
}
