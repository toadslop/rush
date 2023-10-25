use std::net::TcpListener;

use actix_web::dev::Server;
use surrealdb::{engine::any::Any, Surreal};

use crate::{
    configuration::{app::ApplicationSettings, Settings},
    database::init::init_db,
    mailer::init_mailer,
    run,
};

pub struct Application {
    port: u16,
    server: Server,
    db: Surreal<Any>,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let ApplicationSettings {
            host,
            port,
            environment,
        } = configuration.application;
        let address = format!("{host}:{port}");

        let db = init_db(configuration.database)
            .await
            .expect("Could not initialize db");
        let mailer = init_mailer(configuration.mail, environment).await;

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, db.clone(), mailer).await?;

        Ok(Self { port, server, db })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn get_db_ref(&self) -> &Surreal<Any> {
        &self.db
    }
}
