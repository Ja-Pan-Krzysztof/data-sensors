

/* Sensor exceptions */
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SensorCode {
    Ok,
    BadCalibration,
    HardwareTimeout,
    VoltageToHigh,
    EchoTimeout,
    GpioError,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SensorResult<T> {
    pub value: T,
    pub code: SensorCode,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OledErorr {
    ResetLinefailed,
    Initializationfailed,
    ClearFailed,
    DrawFailed,
    FlushFailed,
}
