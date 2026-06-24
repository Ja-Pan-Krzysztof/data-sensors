from stream import read_sensor
import serial


PORT = '/dev/ttyACM0'
BAUD_RATE = 115200


if __name__ == '__main__':
    with serial.Serial(PORT, BAUD_RATE, timeout=1) as ser:
        while True:
            packet = read_sensor(ser)

            if packet:
                print(packet)

            else:
                print("None")
