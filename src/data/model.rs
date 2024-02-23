use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use utoipa::{IntoParams, ToResponse, ToSchema};

#[derive(Debug, Deserialize, Serialize, ToResponse, ToSchema)]
pub struct Accelerometer {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Deserialize, Serialize, ToResponse, ToSchema)]
pub struct Gps {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize, Serialize, ToResponse, ToSchema)]
pub struct Agent {
    pub accelerometer: Accelerometer,
    pub gps: Gps,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, ToResponse, ToSchema)]
pub struct ProcessedAgent {
    #[serde(flatten)]
    pub agent_data: Agent,
    #[schema(max_length = 255, example = "NORMAL")]
    pub road_state: String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Deserialize,
    Serialize,
    sqlx::Type,
    IntoParams,
)]
#[repr(transparent)]
#[serde(transparent)]
#[sqlx(transparent)]
#[into_params(names("id"))]
/// ID of the processed agent to read, update, or delete.
pub struct ProcessedAgentId(i32);

pub trait Dto {
    type Id<'a>;
}

impl Dto for ProcessedAgent {
    type Id<'a> = ProcessedAgentId;
}

impl Dto for [ProcessedAgent] {
    type Id<'a> = &'a [ProcessedAgentId];
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessedAgentDao {
    pub road_state: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
}

impl Display for ProcessedAgentId {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<ProcessedAgent> for ProcessedAgentDao {
    fn from(agent: ProcessedAgent) -> Self {
        Self {
            road_state: agent.road_state,
            x: agent.agent_data.accelerometer.x,
            y: agent.agent_data.accelerometer.y,
            z: agent.agent_data.accelerometer.z,
            latitude: agent.agent_data.gps.latitude,
            longitude: agent.agent_data.gps.longitude,
            timestamp: agent.agent_data.timestamp,
        }
    }
}

impl From<ProcessedAgentDao> for ProcessedAgent {
    fn from(dao: ProcessedAgentDao) -> Self {
        Self {
            agent_data: Agent {
                accelerometer: Accelerometer {
                    x: dao.x,
                    y: dao.y,
                    z: dao.z,
                },
                gps: Gps {
                    latitude: dao.latitude,
                    longitude: dao.longitude,
                },
                timestamp: dao.timestamp,
            },
            road_state: dao.road_state,
        }
    }
}
