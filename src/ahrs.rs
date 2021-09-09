use crate::config::Ahrs as AhrsConfig;
use ahrs::Ahrs;
/// AHRS processing module.
use hdcomm_core::stream::AhrsBody;
use nalgebra::{Matrix1x3, Vector3};

/// Scaled AHRS sample.
#[derive(PartialEq, Debug, Clone)]
pub struct Sample {
    /// Sample timestamp recorded by the device.
    ///
    /// In units of seconds.
    pub timestamp: f64,
    /// Scaled accelerometer reading.
    ///
    /// In units of ms^-2.
    pub acc: Vector3<f64>,
    /// Scaled gyroscope reading.
    ///
    /// In units of rads^-1.
    pub gyro: Vector3<f64>,
    /// Scaled & corrected magnetometer reading.
    ///
    /// In units of Tesla.
    ///
    /// y & x are swapped to correct for the misalignment within the sensor
    /// package of the magnetometer's axes with those of the accelerometer and
    /// gyroscope.
    pub mag: Vector3<f64>,
}

impl Sample {
    /// Create a scaled sample from a raw sample.
    pub fn new(config: &AhrsConfig, raw: &AhrsBody) -> Self {
        let acc = Vector3::from_iterator(raw.acc.iter().map(|a| *a as f64 * config.acc_lsb));
        let gyro = Vector3::from_iterator(raw.gyro.iter().map(|g| *g as f64 * config.gyro_lsb));
        let mag = {
            let mut out =
                Matrix1x3::from_iterator(raw.mag.iter().map(|m| *m as f64 * config.mag_lsb));
            out -= config.hard_iron_correction();
            out *= config.soft_iron_correction();
            out.swap((0, 0), (0, 1));
            out.transpose()
        };
        Self {
            timestamp: raw.time_ms as f64 / 1e3,
            acc,
            gyro,
            mag,
        }
    }
}

/// Euler angles.
///
/// All angles are in degrees.
#[derive(PartialEq, Debug, Clone)]
pub struct Angles {
    /// Device timestamp in seconds.
    pub timestamp: Option<f64>,
    /// Pitch
    pub pitch: f64,
    /// Roll
    pub roll: f64,
    /// Yaw
    pub yaw: f64,
}

/// Simple radians -> degrees conversion.
fn rad2deg(rad: f64) -> f64 {
    360.0 * (rad / (2.0 * std::f64::consts::PI))
}

/// AHRS sensor fusion filter.
pub struct Filter {
    /// Filtering configuration.
    config: AhrsConfig,
    /// Internal filter.
    filter: ahrs::Madgwick<f64>,
    /// Last update time as measured by the device.
    last_update_time_device: Option<f64>,
}

impl Filter {
    /// Create a sensor fusion filter.gyroscope
    pub fn new(config: &AhrsConfig) -> Self {
        Self {
            config: config.clone(),
            filter: ahrs::Madgwick::new(1.0 / config.sampling_rate, config.beta),
            last_update_time_device: None,
        }
    }

    /// Update the filter with a new raw sensor reading.
    pub fn update(&mut self, raw: &AhrsBody) {
        let sample = Sample::new(&self.config, raw);
        self.filter.update_imu(&sample.gyro, &sample.acc).unwrap();
        //self.filter.update(&sample.gyro, &sample.acc, &sample.mag).unwrap();
        self.last_update_time_device = Some(sample.timestamp);
    }

    /// Obtain the euler angles associated with the currently tracked
    /// orientation.
    ///
    /// The device timestamp is also provided. May be `None` when there was
    /// no updates.
    pub fn euler_angles(&self) -> Angles {
        let (roll, pitch, yaw) = self.filter.quat.euler_angles();
        Angles {
            timestamp: self.last_update_time_device,
            pitch: rad2deg(pitch),
            roll: rad2deg(roll),
            yaw: rad2deg(yaw),
        }
    }
}
