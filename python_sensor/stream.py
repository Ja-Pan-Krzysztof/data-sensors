import serial
import json


def read_sensor(s: serial.Serial) -> dict | None:
    """Listens to hte UART seiral port for incoming JSON packets from ESP32

    :return: dict of data sernsor | None
    """

    if s.in_waiting > 0:
        raw_data = s.readline().decode('utf-8', errors='ignore').strip()

        if raw_data.startswith('{') and raw_data.endswith('}'):
            try:
                return json.loads(raw_data)

            except json.JSONDecodeError:
                return None

    return None
