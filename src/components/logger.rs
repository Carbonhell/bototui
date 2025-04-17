use aws_config::SdkConfig;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::info;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tui_logger::{TuiLoggerWidget, TuiWidgetState, TuiWidgetEvent};

use super::Component;
use crate::{action::Action, config::Config};
use crate::action::LoggerAction;

#[derive(Default)]
pub struct Logger {
    command_tx: Option<UnboundedSender<Action>>,
    aws_config: Option<SdkConfig>,
    config: Config,
    state: TuiWidgetState,
    scroll_offset: usize,
}

impl Logger {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Logger {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn register_aws_config_handler(&mut self, config: SdkConfig) -> Result<()> {
        self.aws_config = Some(config);
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::PageUp => Ok(Some(Action::TuiLogger(LoggerAction::PrevPageKey))),
            KeyCode::PageDown => Ok(Some(Action::TuiLogger(LoggerAction::NextPageKey))),
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            Action::TuiLogger(logger_action) => {
                info!("Received logger action: {:?}", logger_action);
                self.state.transition(logger_action.into());
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::default()
            .title(format!("Logs (Use ↑↓ to scroll, offset: {})", self.scroll_offset))
            .borders(Borders::ALL);

        // Create the logger widget with state
        let logger_widget = TuiLoggerWidget::default()
            .block(block)
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Gray))
            .style_info(Style::default().fg(Color::Blue))
            .output_separator(' ')  // Use space as separator
            .output_timestamp(Some("%H:%M:%S".to_string()))  // Show timestamp in HH:MM:SS format
            .output_level(Some(tui_logger::TuiLoggerLevelOutput::Abbreviated))  // Show abbreviated level
            .output_target(false)  // Don't show target
            .output_file(false)  // Don't show file
            .output_line(false)  // Don't show line
            .state(&self.state);

        // Render the logger widget
        frame.render_widget(logger_widget, area);

        Ok(())
    }
}
