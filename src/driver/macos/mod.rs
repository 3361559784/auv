use super::Driver;
use crate::model::{
  AuvResult, DriverCall, DriverDescriptor, DriverResponse, ProducedArtifact, now_millis,
};

mod constants;
mod control;
mod descriptor;
mod dispatch;
mod observe;
mod support;
#[cfg(test)]
mod tests;

mod types;

pub(crate) use self::constants::*;
pub(crate) use self::support::*;
pub(crate) use self::types::*;

pub(crate) struct MacOsObserveDriver;
