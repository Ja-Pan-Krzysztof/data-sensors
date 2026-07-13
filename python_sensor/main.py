import time
import serial

from typing import List

from stream import read_sensor
from models import SensorStat, SessionLocal
import crud


PORT = '/dev/ttyACM0'
BAUD_RATE = 115200


class SensorDataBuffer:
    """RAM buffer menager. Optimises number of I/O operations."""

    def __init__(self, session_factory, limit: int = 10):
        """
        :param session_factory: Instance of models sessionsmaker
        :param limit: Once this value has been entered, results will be saved automativally to db
        """

        self.buffer: List[SensorStat] = []
        self.SessionFactory = session_factory
        self.limit = limit

    def add_reading(self, packet: dict):
        """Convert to ready ORM object and add to buffor"""

        reading = SensorStat(
            temperature=packet['temp'],
            light=packet['light'],
            is_tilted=packet['tilted'],
            is_shocked=packet['shock'],
            distance=packet['dist']
        )
        self.buffer.append(reading)

        if len(self.buffer) >= self.limit:
            self._flush_to_db()

    def _flush_to_db(self):
        """Saving data from buffor to database"""

        if not self.buffer:
            return None

        with self.SessionFactory() as db:
            try:
                crud.save_readings(db, self.buffer)
                print(f'[DB] -> Successfully saved to db')
                self.buffer.clear()

            except Exception as e:
                db.rollback()
                print(f'[DB ERROR] -> Write error: {e}')


if __name__ == '__main__':
    data_menager = SensorDataBuffer(session_factory=SessionLocal)

    with serial.Serial(PORT, BAUD_RATE, timeout=1) as ser:
        try:
            while True:
                p = read_sensor(ser)

                if p:
                    alarms: dict = p['alarms']
                    active_alarms = [i.upper() for i, active in alarms.items() if active]

                    print(f'[ALARM ACTIVE] -> Breached thresholds: {", ".join(active_alarms)}')
                    print(f"[SYSTEM OK] Temp: {p['temp']:.2f}°C | Light: {p['light']:.1f}% | Dist: {p['dist']:.1f}cm")

                    data_menager.add_reading(p)

                time.sleep(0.01)

        except KeyboardInterrupt:
            print('STOP')
