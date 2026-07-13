import serial
import struct

PACKET_SIZE = 14


def read_sensor(s: serial.Serial) -> dict | None:
    """Listens to hte UART seiral port for incoming JSON packets from ESP32

    :return: dict of data sernsor | None
    """

    # Clear old data from buffer if there is too much
    if s.in_waiting > 100:
        s.reset_input_buffer()

        return None

    if s.in_waiting >= PACKET_SIZE:
        try:
            raw_bytes = s.read(PACKET_SIZE)
            temp, light, raw_status, alarm_mask, dist = struct.unpack('<ffBBf', raw_bytes)

            # Protection against data corrupted
            if abs(temp) > 200 or abs(light) > 200 or abs(dist) > 5000:
                s.reset_input_buffer()

                return None

            tilted = bool(raw_status & 1)
            shock = bool((raw_status >> 1) & 1)

            return {
                'temp': round(temp, 2),
                'light': round(light, 2),
                'tilted': bool(tilted),
                'shock': bool(shock),
                'dist': round(dist, 2),
                'alarms': {
                    'temperature': bool(alarm_mask & (1 << 0)),
                    'light': bool(alarm_mask & (1 << 1)),
                    'tilt': bool(alarm_mask & (1 << 2)),
                    'shock': bool(alarm_mask & (1 << 3)),
                    'distance': bool(alarm_mask & (1 << 4))
                }
            }

        except (struct.error, serial.SerialException):
            return None

    return None
