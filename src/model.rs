/// Robot model and move generator.
use crate::config::{Model as ModelConfig, Motion as MotionConfig};
use hdcomm_core::rpc::MoveReqBody;
use s_curve::{SCurveConstraints, SCurveInput, SCurveParameters, SCurveStartConditions};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum ModelError {
    #[error("turn radius not supported")]
    RadiusNotSupported,
    #[error("motion profile limits must be positive")]
    ProfileLimitsNonPositive,
}

/// Model models the nanocar robot and generates actual physical moves from
/// abstract move commands.
pub struct Model {
    /// Robot model configuration.
    pub model: ModelConfig,
    /// Motion configuration.
    pub motion: MotionConfig,
}

impl Model {
    /// Generate a move request for a given turn `radius` (in metres) and move
    /// `distance` (also in metres).
    ///
    /// The radius is indexed and is negative for right turns, and positive for
    /// left turns.
    ///
    /// A radius of zero means a straight move.
    ///
    /// Both `radius` and `distance` are measured from the center of mass of
    /// the robot.
    ///
    /// `steering_setup_ms` will be clamped to `0xffff` if the value specified
    /// in `steering_setup_time` exceeds `0xffff` milliseconds.
    pub fn generate_move(&self, radius: i32, distance: f64) -> Result<MoveReqBody, ModelError> {
        let ref_left = radius >= 0;
        let straight = radius == 0;
        let reverse = distance < 0.;

        let (ratio, ref_ticks, steering) = if straight {
            (
                1.,
                distance * self.model.counts_per_metre,
                self.model.neutral_control,
            )
        } else {
            match self.model.turn_radii.get(radius.abs() as usize) {
                Some(r) => {
                    let center_radius = r.radius;
                    let control = r.control;

                    let ref_radius = center_radius + (self.model.w / 2.);
                    let follower_radius = center_radius - (self.model.w / 2.);
                    let ratio = follower_radius / ref_radius;

                    let ticks =
                        (distance * (ref_radius / center_radius)) * self.model.counts_per_meter;

                    (ratio, ticks, control)
                }
                None => return Err(ModelError::RadiusNotSupported),
            }
        };

        let steering_setup_ms = (self.motion.steering_setup_time * 1e3) as u16;

        let constraints = SCurveConstraints {
            max_acceleration: self.motion.max_accel as f32,
            max_jerk: self.motion.max_jerk as f32,
            max_velocity: self.motion.max_velocity as f32,
        };

        let start_conditions = SCurveStartConditions {
            q0: 0.,
            q1: ref_ticks as f32,
            v0: 0.,
            v1: 0.,
        };

        let input = SCurveInput {
            constraints,
            start_conditions,
        };

        let time_intervals = input.calc_intervals();
        let params = SCurveParameters::new(&time_intervals, &input);

        Ok(MoveReqBody {
            params,
            ratio: ratio as f32,
            ref_left,
            steering: steering as f32,
            steering_setup_ms,
            reverse,
        })
    }

    pub fn set_motion_profile_limits(
        &mut self,
        max_jerk: f64,
        max_accel: f64,
        max_velocity: f64,
    ) -> Result<(), ModelError> {
        if (max_jerk <= 0.) || (max_accel <= 0.) || (max_velocity <= 0.) {
            return Err(ModelError::ProfileLimitsNonPositive);
        }

        self.motion.max_jerk = max_jerk;
        self.motion.max_accel = max_accel;
        self.motion.max_velocity = max_velocity;

        Ok(())
    }
}
