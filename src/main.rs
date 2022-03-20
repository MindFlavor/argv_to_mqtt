use rand::Rng;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use std::time::SystemTime;

fn main() {
    env_logger::init();

    let start_time = SystemTime::now();

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
    let mqttoptions = MqttOptions::new(client_name, "dns.mindflavor.it", 1883);
    let (mut client, mut connection) = Client::new(mqttoptions, 5);

    client
        .publish(
            format!("argv_to_mqtt/{}", topic),
            QoS::ExactlyOnce,
            false,
            values_str,
        )
        .expect("failed to send MQTT message");

    for notification in connection.iter() {
        match notification {
            Err(error) => {
                log::error!("{}", error);
                log::debug!("processing took {:?}", start_time.elapsed());
                panic!();
            }
            Ok(res) => {
                log::debug!("{:?}", res);
                match res {
                    Event::Incoming(Packet::PubComp(_)) => {
                        log::debug!("processing took {:?}", start_time.elapsed());
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
