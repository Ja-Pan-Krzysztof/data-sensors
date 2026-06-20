import serial
import json
import time


PORT = '/dev/ttyACM0'
BAUD_RATE = 115200


def listen():
    """Listens to hte UART seiral port for incoming JSON packets from ESP32

    :return: None
    """
    print(f'[*] Waiting for connection from ESP32 on port {PORT}')

    while True:
        try:
            with serial.Serial(PORT, BAUD_RATE, timeout=2) as s:
                print('Successfully connected')

                s.reset_input_buffer()  # Clean pipe

                while True:
                    if s.in_waiting > 0:
                        raw_data = s.readline().decode('utf-8', errors='ignore').strip()

                        if raw_data.startswith('{') and raw_data.endswith('}'):
                            try:
                                payload = json.loads(raw_data)
                                print(payload)
                                #print(f"[{time.strftime('%H:%M:%S')}] Temp: {payload['temp']:>5.1f}°C | Światło: {payload['light']:>5.1f}% | Dystans: {payload['dist']:>5.1f}cm | Wstrząs: {payload['shock']}")

                            except json.JSONDecodeError:
                                print(f'Dropped corrupted packet: {raw_data}')

        except serial.SerialException:
            print(f'Device not found on {PORT}')
            time.sleep(2)

        except KeyboardInterrupt:
            print('\nListen stopped by user')

            break


if __name__ == '__main__':
    listen()
