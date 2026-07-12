from datetime import datetime, timezone

from sqlalchemy import create_engine, DateTime, Float, Boolean
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, sessionmaker

DATABASE_URL = 'sqlite:///sensor_data.db'
engine = create_engine(DATABASE_URL, echo=True)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)


class Base(DeclarativeBase):
    pass


class SensorStat(Base):
    """Main database for saving values transmitted from sensors"""

    __tablename__ = 'sensor_stats'

    id: Mapped[int] = mapped_column(primary_key=True)
    temperature: Mapped[float] = mapped_column(Float, nullable=False)
    light: Mapped[float] = mapped_column(Float, nullable=False)
    is_tilted: Mapped[bool] = mapped_column(Boolean, nullable=False)
    is_shocked: Mapped[bool] = mapped_column(Boolean, nullable=False)
    distance: Mapped[float] = mapped_column(Float, nullable=False)
    timestamp: Mapped[datetime] = mapped_column(DateTime, default=lambda: datetime.now(timezone.utc))

    def __repr__(self) -> str:
        return (f"<SensorStat(id={self.id}, temp='{self.temperature}', light={self.light}, tilted={self.is_tilted}, "
                f"shock={self.is_shocked}, distance={self.distance}, time={self.timestamp})>")


Base.metadata.create_all(bind=engine)
