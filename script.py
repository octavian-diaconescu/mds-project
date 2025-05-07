from awscrt import io, mqtt
from awsiot import mqtt_connection_builder
import time
import json

ENDPOINT = "your_endpoint"
CLIENT_ID = "your_client_id"
PATH_TO_CERT = "./certs/certificate.pem.crt"
PATH_TO_KEY = "./certs/private.pem.key"
PATH_TO_ROOT = "./certs/AmazonRootCA1.pem"
TOPIC = "your_topic"

def create_mqtt_connection():
    event_loop_group = io.EventLoopGroup(1)
    host_resolver = io.DefaultHostResolver(event_loop_group)
    client_bootstrap = io.ClientBootstrap(event_loop_group, host_resolver)

    mqtt_connection = mqtt_connection_builder.mtls_from_path(
        endpoint=ENDPOINT,
        cert_filepath=PATH_TO_CERT,
        pri_key_filepath=PATH_TO_KEY,
        client_bootstrap=client_bootstrap,
        client_id=CLIENT_ID,
        clean_session=False,
        keep_alive_sec=30
    )

    print(f"Connecting to {ENDPOINT}...")
    connect_future = mqtt_connection.connect()
    connect_future.result()
    print("Connected!")
    return mqtt_connection

def simulate_sensor_data():
    return {
        "device_id": CLIENT_ID,
        "temperature": round(20 + (time.time() % 30),2),
        "timestamp": int(time.time())
    }

def main():
    mqtt_connection = create_mqtt_connection()
    try:
        while True:
            data = simulate_sensor_data()
            mqtt_connection.publish(
                topic=TOPIC,
                payload=json.dumps(data),
                qos=mqtt.QoS.AT_LEAST_ONCE
            )
            print(f"Published: {data}")
            time.sleep(10)
    except KeyboardInterrupt:
        print("Disconnecting...")
        mqtt_connection.disconnect().result()

if __name__ == "__main__":
    main()