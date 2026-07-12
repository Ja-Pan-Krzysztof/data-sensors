from models import SensorStat

from typing import List
from sqlalchemy.orm import Session


def save_readings(db: Session, date_list: List[SensorStat]) -> None:
    """Got ready-made list of objects and saving all data at once"""
    db.add_all(date_list)
    db.commit()
