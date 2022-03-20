use rand::Rng;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use std::{path::Path, time::SystemTime};

#[derive(Debug, Deserialize)]
struct Config {
    pub host: String,
    pub port: Option<u16>,
}

fn main() {
    env_logger::init();

    let start_time = SystemTime::now();

    // read config from file
    let config: Config = {
        let mut paths = Vec::new();
        if let Ok(home_config) =
            std::env::var("HOME").map(|home| home + "/.config/argv_to_mqtt/config.toml")
        {
            paths.push(home_config);
        }
        paths.push("/etc/argv_to_mqtt/config.toml".to_owned());

        let file = paths
            .iter()
            .find(|item| Path::new(item).exists())
            .unwrap_or_else(|| panic!("no valid configuration file found in {:?}", paths));

        log::debug!("about to read toml config file {:?}", file);
        let file_contents = std::fs::read_to_string(file)
            .expect("failure to read from config file (permission error?)");

        log::debug!("about to parse toml config file:\n{}", file_contents);
        toml::from_str(&file_contents).unwrap()
    };
    log::info!("using configuration {:#?}", config);

    let mut iterator = std::env::args();
    let program = iterator.next().unwrap();
    let topic = format!(
        "argv_to_mqtt/{}",
        iterator.next().expect("topic not supplied (arg 1)")
    );
    let values = iterator.collect::<Vec<_>>();

    log::debug!(
        "program == {}, topic == {}, values == {:?}",
        program,
        topic,
        values
    );

    let values_str = serde_json::to_string(&values).expect("cannot convert string array to json");
    log::debug!("values_str == {}", values_str);

    // Create a client & define connect options
    let mut rng = rand::thread_rng();
    let client_name = format!("argv_to_mqtt_{}", rng.gen::<u32>());
    let mqttoptions = MqttOptions::new(client_name, config.host, config.port.unwrap_or(1883));
    let (mut client, mut connection) = Client::new(mqttoptions, 5);

    client
        .publish(topic, QoS::ExactlyOnce, false, values_str)
        .expect("failed to send MQTT message");

    for notification in connection.iter() {
        match notification {
            Err(error) => {
                log::error!("{}", error);
                log::info!("processing took {:?}", start_time.elapsed());
                panic!();
            }
            Ok(res) => {
                log::debug!("{:?}", res);
                match res {
                    Event::Incoming(Packet::PubComp(_)) => {
                        log::info!("processing took {:?}", start_time.elapsed());
                        return;
                    }
                    _ => {
                        log::trace!("ignoring event");
                    }
                }
            }
        }
    }
}
