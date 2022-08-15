use async_std::{io::WriteExt, net::TcpListener, stream::StreamExt};
use futures::future::join_all;
use lci_gateway::{GeneratorState, HvacFan, HvacMode};

#[tokio::main]
async fn main() {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:8888")
        .await
        .expect("Failed to bind to 0.0.0.0:8888");
    log::trace!("Listener started");
    log::info!("Listening on 0.0.0.0:8888");
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        match respond(stream).await {
            Ok(()) => log::trace!("Sucessful respond."),
            Err(e) => log::error!("Failed to respond: {:?}", e),
        };
    }
}

#[derive(Debug)]
enum RespondError {
    ThingError(lci_gateway::ThingError),
    UnknownError,
    IoError(std::io::Error),
}

async fn respond(
    stream: Result<async_std::net::TcpStream, std::io::Error>,
) -> Result<(), RespondError> {
    let mut stream = stream?;
    log::trace!("Trying to get things");
    let things = lci_gateway::get_things().await?;
    log::trace!("Have things and stream.");

    let metrics: String = join_all(things.into_iter().map(|thing| async move {
        match thing.get_type() {
            Some(lci_gateway::DeviceType::Tank) => Some(log_tank(thing).await),
            Some(lci_gateway::DeviceType::Hvac) => Some(log_hvac(thing).await),
            Some(lci_gateway::DeviceType::Generator) => Some(log_generator(thing).await),
            _ => None,
        }
    }))
    .await
    .into_iter()
    .filter_map(|row| match row {
        Some(row) => Some(row.unwrap()),
        None => None,
    })
    .collect::<Vec<String>>()
    .join("\n");

    let headers = [
        "HTTP/1.1 200 OK".to_string(),
        "Connection: close".to_string(),
        "Content-Type: text/plain".to_string(),
        format!("Content-Length: {}", metrics.len()),
    ]
    .join("\r\n");
    let response = format!("{}\r\n\r\n{}", headers, metrics);
    match stream.write(response.as_bytes()).await {
        Ok(_) => log::trace!(
            "Sent metrics on HTTP request. Response size {}",
            response.len()
        ),
        Err(e) => log::error!("Error sending metrics, {}", e),
    };

    Ok(())
}

async fn log_tank(thing: lci_gateway::Thing) -> Result<String, ()> {
    log::trace!("Building tank response for {}", thing.label());
    let normalized = thing.label().replace(" ", "_").to_lowercase();
    let tank = lci_gateway::Tank::new(thing)?;
    let field = format!("lci_gateway_{normalized}");
    let mut buffer = "".to_string();

    let help = format!("# HELP {field} Tank percentage\n");
    buffer.push_str(&help);
    let metric_type = format!("# TYPE {field} gauge\n");
    buffer.push_str(&metric_type);
    let value = format!("{field} {value}\n", value = tank.level().await);
    buffer.push_str(&value);
    log::debug!("Responding with built tank for {}", tank.label());
    Ok(buffer)
}

async fn log_hvac(thing: lci_gateway::Thing) -> Result<String, ()> {
    log::trace!("Building hvac response for {}", thing.label());
    let normalized = thing.label().replace(" ", "_").to_lowercase();
    let hvac = lci_gateway::HVAC::new(thing)?;
    let field_base = format!("lci_gateway_{normalized}");
    let mut buffer = "".to_string();

    let field = format!("{field_base}_outside_temprature");
    let value = hvac.outside_temprature().await;
    let add = format!("# HELP {field} HVAC outside temprature\n");
    buffer.push_str(&add);
    let add = format!("# TYPE {field} gauge\n");
    buffer.push_str(&add);
    let add = format!("{field} {value}\n");
    buffer.push_str(&add);

    let field = format!("{field_base}_inside_temprature");
    let value = hvac.inside_temprature().await;
    let add = format!("# HELP {field} HVAC inside temprature\n");
    buffer.push_str(&add);
    let add = format!("# TYPE {field} gauge\n");
    buffer.push_str(&add);
    let add = format!("{field} {value}\n");
    buffer.push_str(&add);

    if let Ok(state) = hvac.fan().await {
        let field = format!("{field_base}_fan");
        // TODO: move this to a proc macro over states. There are 18 values for state.
        let add = format!(
            "# HELP {field} Fan state. Auto = {}, Low = {}, High = {}\n",
            HvacFan::Auto as u8,
            HvacFan::Low as u8,
            HvacFan::High as u8
        );
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let state = state as u8;
        let add = format!("{field} {state}\n");
        buffer.push_str(&add);
    }

    if let Ok(state) = hvac.mode().await {
        let field = format!("{field_base}_mode");
        let add = format!(
            "# HELP {field} A/C mode. Off = {}, Heat = {}, Cool = {}, Heat-Cool = {}\n",
            HvacMode::Off as u8,
            HvacMode::Heat as u8,
            HvacMode::Cool as u8,
            HvacMode::HeatCool as u8
        );
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let state = state as u8;
        let add = format!("{field} {state}\n");
        buffer.push_str(&add);
    }

    // TODO: hvac.status()

    log::debug!("Responding with built hvac for {}", hvac.label());
    Ok(buffer)
}

async fn log_generator(thing: lci_gateway::Thing) -> Result<String, ()> {
    log::trace!("Building generator response for {}", thing.label());
    let normalized = thing.label().replace(" ", "_").to_lowercase();
    let generator = lci_gateway::Generator::new(thing)?;
    let field = format!("lci_gateway_{normalized}_state");
    let mut buffer = "".to_string();

    if let Ok(state) = generator.state().await {
        let add = format!(
            "# HELP {field} Generator state. Off = {}, Priming = {}, Starting = {}, Running = {}\n",
            GeneratorState::Off as u8,
            GeneratorState::Priming as u8,
            GeneratorState::Starting as u8,
            GeneratorState::Running as u8
        );
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let add = format!("{field} {}\n", state as u8);
        buffer.push_str(&add);
        log::debug!("Responding with built generator for {}", generator.label());
    } else {
        log::debug!("Not enough information to build {}", generator.label());
    }

    Ok(buffer)
}

impl From<lci_gateway::ThingError> for RespondError {
    fn from(error: lci_gateway::ThingError) -> Self {
        RespondError::ThingError(error)
    }
}

impl From<std::io::Error> for RespondError {
    fn from(error: std::io::Error) -> Self {
        RespondError::IoError(error)
    }
}

impl From<()> for RespondError {
    fn from(_error: ()) -> Self {
        RespondError::UnknownError
    }
}
