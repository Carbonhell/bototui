use serde::{Deserialize, Serialize};
use strum::Display;
use tui_logger::TuiWidgetEvent;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    // tui-logger pagination actions
    TuiLogger(LoggerAction),
    // Navigation actions
    NavigateToHome,
    NavigateToBatch,
    NavigateToLogger,
    Back,
    // Batch actions
    UpdateBatchJobs(Vec<crate::components::batch::JobInfo>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Display)]
pub enum LoggerAction {
    PrevPageKey,
    NextPageKey,
}

impl From<LoggerAction> for TuiWidgetEvent {
    fn from(action: LoggerAction) -> Self {
        match action {
            LoggerAction::PrevPageKey => TuiWidgetEvent::PrevPageKey,
            LoggerAction::NextPageKey => TuiWidgetEvent::NextPageKey,
        }
    }
}
