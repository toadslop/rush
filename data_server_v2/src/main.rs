use rush_data_server::{
    configuration::get_configuration, startup::Application, telemetry::init_telemetry,
};
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    init_telemetry()?;
    let configuration = get_configuration().expect("Failed to read configuration");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
