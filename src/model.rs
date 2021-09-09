/// Robot model and move generator.
use crate::config::{Model as ModelConfig, Motion as MotionConfig};
use hdcomm_core::rpc::MoveReqBody;
use s_curve::{SCurveConstraints, SCurveInput, SCurveParameters, SCurveStartConditions};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum Error {
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
    /// The radius is indexed and is negative for turns with the turn center
    /// being on the right side of the robot, and positive for turns with the
    /// turn center being on the left side of the robot.
    ///
    /// A radius of zero means a straight move.
    ///
    /// The distance to move is measured from the center of mass of
    /// the robot.
    ///
    /// `steering_setup_ms` will be clamped to `0xffff` if the value specified
    /// in `steering_setup_time` exceeds `0xffff` milliseconds.
    pub fn generate_move(&self, radius: i32, distance: f64) -> Result<MoveReqBody, Error> {
        // True for right turns, false for left turns.
        let ref_left = radius <= 0;
        let left_turn = !ref_left;
        let straight = radius == 0;
        let reverse = distance < 0.;

        let (ratio, ref_ticks, steering) = if straight {
            (
                1.,
                distance * self.model.counts_per_metre,
                self.model.neutral_control,
            )
        } else {
            match self
                .model
                .turn_radii
                .get((radius.abs() as usize).saturating_sub(1))
            {
                Some(r) => {
                    let center_radius = r.radius;
                    let control = if left_turn {
                        r.control_left
                    } else {
                        r.control_right
                    };

                    let ref_radius = center_radius + (self.model.w / 2.);
                    let follower_radius = center_radius - (self.model.w / 2.);
                    let ratio = follower_radius / ref_radius;

                    let ticks =
                        (distance * (ref_radius / center_radius)) * self.model.counts_per_metre;

                    (ratio, ticks, control)
                }
                None => return Err(Error::RadiusNotSupported),
            }
        };

        // This is a saturating conversion to u16.
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
    ) -> Result<(), Error> {
        if (max_jerk <= 0.) || (max_accel <= 0.) || (max_velocity <= 0.) {
            return Err(Error::ProfileLimitsNonPositive);
        }

        self.motion.max_jerk = max_jerk;
        self.motion.max_accel = max_accel;
        self.motion.max_velocity = max_velocity;

        Ok(())
    }
}
