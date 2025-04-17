use aws_config::SdkConfig;
use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    aws_config: Option<SdkConfig>,
    config: Config,
    selected_service: usize,
    services: Vec<Service>,
    list_state: ListState,
}

impl Default for Home {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            command_tx: None,
            aws_config: None,
            config: Config::default(),
            selected_service: 0,
            services: Vec::new(),
            list_state,
        }
    }
}

#[derive(Clone)]
struct Service {
    name: String,
    description: String,
    action: Action,
}

impl Home {
    pub fn new() -> Self {
        Home {
            services: vec![
                Service {
                name: "AWS Batch".to_string(),
                description: "Manage AWS Batch jobs and job queues".to_string(),
                action: Action::NavigateToBatch,
            },
            Service {
                name: "Logs".to_string(),
                description: "View application logs".to_string(),
                action: Action::NavigateToLogger,
            },
            // More services can be added here as they are implemented
            ],
            ..Self::default()
        }
    }
}

impl Component for Home {
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
        use crossterm::event::KeyCode;

        match key.code {
            // Navigate up in the list
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.services.is_empty() {
                    self.selected_service = self.selected_service.saturating_sub(1);
                    self.list_state.select(Some(self.selected_service));
                }
                Ok(Some(Action::Render))
            }
            // Navigate down in the list
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.services.is_empty() {
                    self.selected_service = (self.selected_service + 1).min(self.services.len() - 1);
                    self.list_state.select(Some(self.selected_service));
                }
                Ok(Some(Action::Render))
            }
            // Select the current service
            KeyCode::Enter => {
                if let Some(service) = self.services.get(self.selected_service) {
                    Ok(Some(service.action.clone()))
                } else {
                    Ok(None)
                }
            }
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
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let title = "Bototui";

        // Create a block for the entire widget
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        // Create a list of services
        let items: Vec<ListItem> = self.services
            .iter()
            .map(|service| {
                ListItem::new(vec![
                    Line::from(format!("{}", service.name)),
                    Line::from(format!("  {}", service.description)),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        // Render the block and list
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        frame.render_stateful_widget(
            list,
            inner_area,
            &mut self.list_state,
        );

        Ok(())
    }
}
