use async_std::{io::WriteExt, net::TcpListener, stream::StreamExt};
use futures::future::join_all;
use lci_gateway::{
    DeviceType, GeneratorState, HvacFan, HvacMode, HvacStatus, OnlineState, SwitchState,
};

// TODO: code cleanup if I'm in here enough.
// Otherwise, I'm sad to say copy and paste is messy,
// but good enough.

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

async fn respond(
    stream: Result<async_std::net::TcpStream, std::io::Error>,
) -> Result<(), RespondError> {
    let mut stream = stream?;
    log::trace!("Trying to get things");
    let things = lci_gateway::get_things().await?;
    log::trace!("Have things and stream.");

    let metrics: String = join_all(things.into_iter().map(|thing| async move {
        match thing.get_type() {
            Some(DeviceType::Tank) => Some(log_tank(thing).await.map_err(|_| ())),
            Some(DeviceType::Hvac) => Some(log_hvac(thing).await.map_err(|_| ())),
            Some(DeviceType::Generator) => Some(log_generator(thing).await.map_err(|_| ())),
            Some(DeviceType::Switch) => Some(log_switch(thing).await.map_err(|_| ())),
            Some(DeviceType::Dimmer) => Some(log_dimmer(thing).await.map_err(|_| ())),
            _ => None,
        }
    }))
    .await
    .into_iter()
    .filter_map(|row| row.map(Result::unwrap))
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

async fn log_tank(thing: lci_gateway::Thing) -> Result<String, lci_gateway::TankError> {
    log::trace!("Building tank response for {}", thing.label());
    let mut buffer = get_online_state(&thing)
        .await
        .unwrap_or_else(|| "".to_string());
    let normalized = thing.label().replace(' ', "_").to_lowercase();
    let tank = lci_gateway::Tank::new(thing)?;
    let field = format!("lci_gateway_{normalized}");

    let res = tank.level().await;
    if let Ok(value) = res {
        let help = format!("# HELP {field} Tank percentage\n");
        buffer.push_str(&help);
        let metric_type = format!("# TYPE {field} gauge\n");
        buffer.push_str(&metric_type);
        let usage = value.value();
        let row = format!("{field} {usage}\n");
        buffer.push_str(&row);
        log::debug!("Responding with built tank for {}", tank.label());
    } else {
        log::warn!("Failed to build tank for {} due to {:?}", tank.label(), res);
    }

    Ok(buffer)
}

async fn log_hvac(thing: lci_gateway::Thing) -> Result<String, lci_gateway::HvacError> {
    log::trace!("Building hvac response for {}", thing.label());
    let mut buffer = get_online_state(&thing)
        .await
        .unwrap_or_else(|| "".to_string());
    let normalized = thing.label().replace(' ', "_").to_lowercase();
    let hvac = lci_gateway::HVAC::new(thing)?;
    let field_base = format!("lci_gateway_{normalized}");

    let res = hvac.outside_temprature().await;
    if let Ok(value) = res {
        let field = format!("{field_base}_outside_temprature");
        let add = format!("# HELP {field} HVAC outside temprature\n");
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let add = format!("{field} {value}\n");
        buffer.push_str(&add);
    } else {
        log::warn!(
            "Faild to build hvac outside temp for {} due to {:?}",
            hvac.label(),
            res
        );
    }

    let res = hvac.inside_temprature().await;
    if let Ok(value) = res {
        let field = format!("{field_base}_inside_temprature");
        let add = format!("# HELP {field} HVAC inside temprature\n");
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let add = format!("{field} {value}\n");
        buffer.push_str(&add);
    } else {
        log::warn!(
            "Faild to build hvac inside temp for {} due to {:?}",
            hvac.label(),
            res
        );
    }

    let res = hvac.fan().await;
    if let Ok(state) = res {
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
    } else {
        log::warn!(
            "Failed to build hvac fan for {} due to {:?}",
            hvac.label(),
            res
        );
    }

    let res = hvac.status().await;
    if let Ok(state) = res {
        let field = format!("{field_base}_status");
        // TODO: move this to a proc macro over states. There are 18 values for state.
        let add = format!(
            "# HELP {field} HVAC state. Off = {}, Idle = {}, Cooling = {}, Heat Pump = {}, Electric Furnace = {}, Gas Furnace = {}, Gas Override = {}, Dead Time = {}, Load Shedding = {}, Fail Off = {}, Fail Idle = {}, Fail Cooling = {}, Fail Heat Pump = {}, Fail Electric Furnace = {}, Fail Gas Furnace = {}, Fail Gas Override = {}, Fail Dead Time = {}, Fail Shedding = {}\n",
            HvacStatus::Off as u8,
            HvacStatus::Idle as u8,
            HvacStatus::Cooling as u8,
            HvacStatus::HeatPump as u8,
            HvacStatus::ElectricFurnace as u8,
            HvacStatus::GasFurnace as u8,
            HvacStatus::GasOverride as u8,
            HvacStatus::DeadTime as u8,
            HvacStatus::LoadShedding as u8,
            HvacStatus::FailOff as u8,
            HvacStatus::FailIdle as u8,
            HvacStatus::FailCooling as u8,
            HvacStatus::FailHeatPump as u8,
            HvacStatus::FailElectricFurnace as u8,
            HvacStatus::FailGasFurnace as u8,
            HvacStatus::FailGasOverride as u8,
            HvacStatus::FailDeadTime as u8,
            HvacStatus::FailShedding as u8,
        );
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let state = state as u8;
        let add = format!("{field} {state}\n");
        buffer.push_str(&add);
    } else {
        log::warn!(
            "Failed to build hvac status for {} due to {:?}",
            hvac.label(),
            res
        );
    }

    let res = hvac.mode().await;
    if let Ok(state) = res {
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
    } else {
        log::warn!(
            "Failed to build hvac mode for {}, due to {:?}",
            hvac.label(),
            res
        );
    }

    let res = hvac.high_temp().await;
    if let Ok(state) = res {
        let field = format!("{field_base}_high_temperature");
        let add =
            format!("# HELP {field} The hottest the A/C is set to allow without operating.\n",);
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let state = state as u8;
        let add = format!("{field} {state}\n");
        buffer.push_str(&add);
    } else {
        log::warn!(
            "Failed to build hvac high_temperature for {}, due to {:?}",
            hvac.label(),
            res
        );
    }

    let res = hvac.low_temp().await;
    if let Ok(state) = res {
        let field = format!("{field_base}_low_temperature");
        let add = format!(
            "# HELP {field} The lowest the furnace or A/C is set to allow without operating.\n",
        );
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let state = state as u8;
        let add = format!("{field} {state}\n");
        buffer.push_str(&add);
    } else {
        log::warn!(
            "Failed to build hvac low_temperature for {}, due to {:?}",
            hvac.label(),
            res
        );
    }

    log::debug!("Responding with built hvac for {}", hvac.label());
    Ok(buffer)
}

async fn log_generator(thing: lci_gateway::Thing) -> Result<String, lci_gateway::GeneratorError> {
    log::trace!("Building generator response for {}", thing.label());
    let mut buffer = get_online_state(&thing)
        .await
        .unwrap_or_else(|| "".to_string());
    let normalized = thing.label().replace(' ', "_").to_lowercase();
    let generator = lci_gateway::Generator::new(thing)?;
    let field = format!("lci_gateway_{normalized}_state");

    let res = generator.state().await;
    if let Ok(state) = res {
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
        log::warn!(
            "Failed to build generator {} due to {:?}",
            generator.label(),
            res
        );
    }

    Ok(buffer)
}

async fn log_switch(thing: lci_gateway::Thing) -> Result<String, lci_gateway::SwitchError> {
    log::trace!("Building switch response for {}", thing.label());
    let mut buffer = get_online_state(&thing)
        .await
        .unwrap_or_else(|| "".to_string());
    let normalized = thing.label().replace(' ', "_").to_lowercase();
    let switch = lci_gateway::Switch::new(thing)?;
    let field_base = format!("lci_gateway_{normalized}");

    let res = switch.relay_current().await;
    if let Ok(value) = res {
        let field = format!("{field_base}_relay_current");
        let add = format!("# HELP {field} Switch relay current\n");
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let add = format!("{field} {value}\n");
        buffer.push_str(&add);
    } else {
        log::info!(
            "Faild to get switch relay current for {} due to {:?}",
            switch.label(),
            res
        );
    }

    let res = switch.state().await;
    if let Ok(state) = res {
        let field = format!("{field_base}_state");
        let add = format!(
            "# HELP {field} Switch state. Off = {}, On = {}\n",
            SwitchState::Off as u8,
            SwitchState::On as u8,
        );
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let add = format!("{field} {}\n", state as u8);
        buffer.push_str(&add);
        log::debug!("Built switch state for {}", switch.label());
    } else {
        log::warn!(
            "Faild to get switch state for {} due to {:?}",
            switch.label(),
            res
        );
    }

    let res = switch.fault().await;
    if let Ok(state) = res {
        let field = format!("{field_base}_fault");
        let add = format!(
            "# HELP {field} Switch state. Off = {}, On = {}\n",
            SwitchState::Off as u8,
            SwitchState::On as u8,
        );
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let add = format!("{field} {}\n", state as u8);
        buffer.push_str(&add);
        log::debug!("Built switch fault state for {}", switch.label());
    } else {
        // Not a warn since not all switches have fault states.
        log::info!(
            "Failed to build switch fault state for {} due to {:?}.",
            switch.label(),
            res
        );
    }

    Ok(buffer)
}

async fn log_dimmer(thing: lci_gateway::Thing) -> Result<String, lci_gateway::DimmerError> {
    log::trace!("Building switch response for {}", thing.label());
    let mut buffer = get_online_state(&thing)
        .await
        .unwrap_or_else(|| "".to_string());
    let normalized = thing.label().replace(' ', "_").to_lowercase();
    let dimmer = lci_gateway::Dimmer::new(thing)?;
    let field_base = format!("lci_gateway_{normalized}");

    let res = dimmer.brightness().await;
    if let Ok(percentage) = res {
        let field = format!("{field_base}_brightness");
        let add = format!("# HELP {field} Dimmer brightness percent, 0 to 100.\n");
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let add = format!("{field} {}\n", percentage.value());
        buffer.push_str(&add);
        log::debug!("Built dimmer brightness for {}", dimmer.label());
    } else {
        log::warn!(
            "Failed to get dimmer brightness for {} due to {:?}.",
            dimmer.label(),
            res
        );
    }

    Ok(buffer)
}

async fn get_online_state(thing: &lci_gateway::Thing) -> Option<String> {
    log::trace!("Building online state for {}", thing.label());
    let res = thing.online().await;
    if let Ok(state) = res {
        let mut buffer = "".to_string();
        let normalized = thing.label().replace(' ', "_").to_lowercase();
        let field = format!("lci_gateway_{normalized}_online");
        let add = format!(
            "# HELP {field} Online state, Offline = {}, Online = {}, Locked = {}\n",
            OnlineState::Offline as u8,
            OnlineState::Online as u8,
            OnlineState::Locked as u8,
        );
        buffer.push_str(&add);
        let add = format!("# TYPE {field} gauge\n");
        buffer.push_str(&add);
        let add = format!("{field} {}\n", state as u8);
        buffer.push_str(&add);
        log::trace!("Built online state for {}", thing.label());
        Some(buffer)
    } else {
        log::warn!(
            "Failed to build online state for {} due to {:?}.",
            thing.label(),
            res
        );
        None
    }
}

#[derive(Debug)]
enum RespondError {
    ThingError(lci_gateway::ThingError),
    IoError(std::io::Error),
}

impl From<lci_gateway::ThingError> for RespondError {
    fn from(error: lci_gateway::ThingError) -> Self {
        log::error!("Thing error = {:?}", error);
        RespondError::ThingError(error)
    }
}

impl From<std::io::Error> for RespondError {
    fn from(error: std::io::Error) -> Self {
        log::error!("STD IO Error = {:?}", error);
        RespondError::IoError(error)
    }
}
