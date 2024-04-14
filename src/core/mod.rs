#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Lambda {
    pub name: String,
    pub arn: String,
    pub runtime: String,
    pub memory: i64,
    pub timeout: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum EventSourceMappingState {
    Enabled,
    Disabled,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) enum EventSourceMapping {
    SQS {
        name: String,
        batch_size: i64,
        state: EventSourceMappingState,
        batch_window: i64,
    },
    EventBridge {
        name: String,
        event_bus_name: String,
        state: EventSourceMappingState,
    },
}

impl EventSourceMapping {
    pub fn name(&self) -> String {
        match self {
            Self::SQS { name, .. } => name.clone(),
            Self::EventBridge { name, .. } => name.clone(),
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            Self::SQS { .. } => "SQS".to_string(),
            Self::EventBridge { event_bus_name, .. } => {
                format!("EventBridge ({})", event_bus_name).to_string()
            }
        }
    }

    pub fn batch_size(&self) -> Option<i64> {
        match self {
            Self::SQS { batch_size, .. } => Some(*batch_size),
            _ => None,
        }
    }

    pub fn state(&self) -> EventSourceMappingState {
        match self {
            Self::SQS { state, .. } => state.clone(),
            Self::EventBridge { state, .. } => state.clone(),
            _ => EventSourceMappingState::Disabled,
        }
    }

    pub fn minimum_batching_window_in_seconds(&self) -> Option<i64> {
        match self {
            Self::SQS {
                batch_window: minimum_batching_window_in_seconds,
                ..
            } => Some(*minimum_batching_window_in_seconds),
            _ => None,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Metric {
    pub name: String,
    pub metric: String,
    pub timestamps: Vec<u64>,
    pub values: Vec<f64>,
}

pub struct SearchState {
    pub lambdas: Vec<Lambda>,
}

pub struct SearchingState {
    pub lambda: Lambda,
}

pub struct DashboardState {
    pub lambda: Lambda,
    pub metrics: Vec<Metric>,
    pub event_source_mappings: Vec<EventSourceMapping>,
}

pub struct ErrorState {
    pub error_message: String,
}

pub enum State {
    Splash,
    Search(SearchState),
    Searching(SearchingState),
    Dashboard(DashboardState),
    Error(ErrorState),
    Quit,
}

pub enum Action {
    Quit,
    Search,
    PerformSearch { lambda: Lambda },
}
