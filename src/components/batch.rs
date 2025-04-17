use aws_config::SdkConfig;
use aws_sdk_batch::{Client as BatchClient, types::JobStatus};
use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use std::time::{Duration, Instant};
use log::{error, info};
use crossterm::event::KeyEvent;
use super::Component;
use crate::{action::Action, config::Config};

// Structure to represent a job with its details
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JobInfo {
    pub job_id: String,
    pub job_name: String,
    pub status: String,
    pub created_at: Option<String>,
}

#[derive(Default)]
pub struct Batch {
    command_tx: Option<UnboundedSender<Action>>,
    aws_config: Option<SdkConfig>,
    batch_client: Option<BatchClient>,
    config: Config,
    jobs: Vec<JobInfo>,
    last_refresh: Option<Instant>,
    refresh_interval: Duration,
    is_loading: bool,
    selected_job_index: Option<usize>,
    show_job_details: bool,
}

impl Batch {
    pub fn new() -> Self {
        Self {
            refresh_interval: Duration::from_secs(30), // Refresh every 30 seconds
            selected_job_index: None,
            show_job_details: false,
            ..Default::default()
        }
    }

    // Initialize the Batch client when AWS config is available
    fn init_client(&mut self) {
        if let Some(config) = &self.aws_config {
            self.batch_client = Some(BatchClient::new(config));
        }
    }

    // Fetch jobs from AWS Batch
    async fn fetch_jobs(&mut self) -> Result<()> {
        if let Some(client) = &self.batch_client {
            self.is_loading = true;

            // Get list of job queues
            let queues_resp = client.describe_job_queues().send().await;
            info!("{:?}", queues_resp);
            let queues_resp = queues_resp?;
            let job_queues = queues_resp.job_queues();

            let mut all_jobs = Vec::new();

            // TODO: merge the list_job calls and improve the ordering
            // For each queue, get the jobs
            for queue in job_queues {
                if let Some(queue_name) = queue.job_queue_name() {
                    // List jobs with RUNNING status
                    let jobs_resp = client
                        .list_jobs()
                        .job_queue(queue_name)
                        .job_status(JobStatus::Running)
                        .send()
                        .await?;

                    // Process job summary list
                    for job in jobs_resp.job_summary_list() {
                        if let (Some(job_id), Some(job_name)) = (job.job_id(), job.job_name()) {
                            let created_at = job.created_at()
                                .map(|timestamp| {
                                    // Check if timestamp is in milliseconds (too large for a reasonable Unix timestamp in seconds)
                                    let adjusted_timestamp = if timestamp > 100_000_000_000 {
                                        // Convert from milliseconds to seconds
                                        timestamp / 1000
                                    } else {
                                        timestamp
                                    };
                                    let datetime = chrono::DateTime::from_timestamp(adjusted_timestamp, 0)
                                        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                });

                            all_jobs.push(JobInfo {
                                job_id: job_id.to_string(),
                                job_name: job_name.to_string(),
                                status: "RUNNING".to_string(),
                                created_at,
                            });
                        }
                    }

                    // Also get SUCCEEDED jobs
                    let submitted_jobs_resp = client
                        .list_jobs()
                        .job_queue(queue_name)
                        .job_status(JobStatus::Succeeded)
                        .send()
                        .await?;

                    for job in submitted_jobs_resp.job_summary_list() {
                        if let (Some(job_id), Some(job_name)) = (job.job_id(), job.job_name()) {
                            let created_at = job.created_at()
                                .map(|timestamp| {
                                    let datetime = chrono::DateTime::from_timestamp_millis(timestamp)
                                        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                });

                            all_jobs.push(JobInfo {
                                job_id: job_id.to_string(),
                                job_name: job_name.to_string(),
                                status: "Succeeded".to_string(),
                                created_at,
                            });
                        }
                    }
                }
            }

            self.jobs = all_jobs;
            self.last_refresh = Some(Instant::now());
            self.is_loading = false;
        }

        Ok(())
    }
}

// TODO generated with AI - untested/not reviewed yet
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockBatchClient;
    use aws_config::SdkConfig;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_batch_component_with_mock_data() {
        // Create a mock config
        let config = MockBatchClient::mock_config();

        // Create a batch component
        let mut batch = Batch::new();

        // Register the mock config
        batch.register_aws_config_handler(config).unwrap();

        // Create a channel for actions
        let (tx, _rx) = mpsc::unbounded_channel();
        batch.register_action_handler(tx).unwrap();

        // Manually add some jobs to test the display
        batch.jobs.push(JobInfo {
            job_id: "job-1".to_string(),
            job_name: "test-job-1".to_string(),
            status: "RUNNING".to_string(),
            created_at: Some("2023-01-01 12:00:00".to_string()),
        });

        batch.jobs.push(JobInfo {
            job_id: "job-2".to_string(),
            job_name: "test-job-2".to_string(),
            status: "SUBMITTED".to_string(),
            created_at: Some("2023-01-02 13:00:00".to_string()),
        });

        // Verify that the jobs were added
        assert_eq!(batch.jobs.len(), 2);
        assert_eq!(batch.jobs[0].job_id, "job-1");
        assert_eq!(batch.jobs[0].job_name, "test-job-1");
        assert_eq!(batch.jobs[0].status, "RUNNING");
        assert_eq!(batch.jobs[0].created_at, Some("2023-01-01 12:00:00".to_string()));

        assert_eq!(batch.jobs[1].job_id, "job-2");
        assert_eq!(batch.jobs[1].job_name, "test-job-2");
        assert_eq!(batch.jobs[1].status, "SUBMITTED");
        assert_eq!(batch.jobs[1].created_at, Some("2023-01-02 13:00:00".to_string()));
    }
}

impl Component for Batch {
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
        self.init_client();
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match key.code {
            // Navigate up in the job list
            KeyCode::Up => {
                if !self.jobs.is_empty() {
                    if let Some(index) = self.selected_job_index {
                        if index > 0 {
                            self.selected_job_index = Some(index - 1);
                        }
                    } else {
                        self.selected_job_index = Some(0);
                    }
                }
                Ok(Some(Action::Render))
            },
            // Navigate down in the job list
            KeyCode::Down => {
                if !self.jobs.is_empty() {
                    if let Some(index) = self.selected_job_index {
                        if index < self.jobs.len() - 1 {
                            self.selected_job_index = Some(index + 1);
                        }
                    } else {
                        self.selected_job_index = Some(0);
                    }
                }
                Ok(Some(Action::Render))
            },
            // Toggle job details view
            KeyCode::Enter => {
                if self.selected_job_index.is_some() {
                    self.show_job_details = !self.show_job_details;
                }
                Ok(Some(Action::Render))
            },
            // Exit job details view
            KeyCode::Esc => {
                if self.show_job_details {
                    self.show_job_details = false;
                    Ok(Some(Action::Render))
                } else {
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // Check if we need to refresh the job list
                let should_refresh = match self.last_refresh {
                    Some(last) => last.elapsed() >= self.refresh_interval,
                    None => true, // First time, so refresh
                };

                if should_refresh && !self.is_loading {
                    // Clone the command_tx to avoid borrowing issues
                    if let Some(tx) = self.command_tx.clone() {
                        // Create a new client for the async task
                        if let Some(client) = &self.batch_client {
                            let client_clone = client.clone();

                            // Spawn a tokio task to fetch jobs
                            tokio::spawn(async move {
                                let mut batch = Batch::default();
                                batch.batch_client = Some(client_clone);

                                if let Err(e) = batch.fetch_jobs().await {
                                    error!("Failed to fetch jobs: {}", e);
                                    // Send error action if fetch fails
                                    let _ = tx.send(Action::Error(format!("Failed to fetch jobs: {}", e)));
                                } else {
                                    // Send the fetched jobs back to the main thread
                                    let _ = tx.send(Action::UpdateBatchJobs(batch.jobs));
                                }
                            });

                            self.is_loading = true;
                        }
                    }
                }
            }
            Action::Render => {
                // Nothing special to do on render
            }
            Action::UpdateBatchJobs(jobs) => {
                // Update the jobs with the ones fetched in the async task
                self.jobs = jobs;
                self.last_refresh = Some(Instant::now());
                self.is_loading = false;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // If we're showing job details and have a selected job, display the details
        if self.show_job_details && self.selected_job_index.is_some() {
            let job_index = self.selected_job_index.unwrap();
            if job_index < self.jobs.len() {
                let job = &self.jobs[job_index];

                let block = Block::default()
                    .title(format!("Job Details: {}", job.job_name))
                    .borders(Borders::ALL);

                let created_at = job.created_at.clone().unwrap_or_else(|| "N/A".to_string());

                let details = vec![
                    Line::from(vec![
                        Span::styled("Job ID: ", Style::default().fg(Color::Yellow)),
                        Span::raw(&job.job_id),
                    ]),
                    Line::from(vec![
                        Span::styled("Job Name: ", Style::default().fg(Color::Yellow)),
                        Span::raw(&job.job_name),
                    ]),
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                        Span::raw(&job.status),
                    ]),
                    Line::from(vec![
                        Span::styled("Created At: ", Style::default().fg(Color::Yellow)),
                        Span::raw(created_at),
                    ]),
                    Line::from(""),
                    // TODO standardize by using q instead
                    Line::from("Press ESC to go back to the job list"),
                ];

                let paragraph = Paragraph::new(details)
                    .block(block)
                    .wrap(Wrap { trim: true });

                frame.render_widget(paragraph, area);
                return Ok(());
            }
        }

        // Otherwise, show the job list
        let title = if self.selected_job_index.is_some() {
            "AWS Batch Jobs (Use ↑↓ to navigate, Enter to view details)"
        } else {
            "AWS Batch Jobs"
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);

        if self.jobs.is_empty() {
            let status_text = if self.is_loading {
                "Loading jobs..."
            } else {
                "No jobs found. Jobs will appear here when available."
            };

            let paragraph = Paragraph::new(status_text)
                .block(block)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, area);
        } else {
            // Create a table to display jobs
            let header_cells = ["Job ID", "Job Name", "Status", "Created At"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
            let header = Row::new(header_cells).style(Style::default().bg(Color::Blue));

            // Create rows from jobs with highlighting for the selected job
            let rows = self.jobs.iter().enumerate().map(|(i, job)| {
                let created_at = job.created_at.clone().unwrap_or_else(|| "N/A".to_string());
                let cells = [
                    Cell::from(job.job_id.clone()),
                    Cell::from(job.job_name.clone()),
                    Cell::from(job.status.clone()),
                    Cell::from(created_at),
                ];

                let style = if Some(i) == self.selected_job_index {
                    Style::default().bg(Color::Gray)
                } else {
                    Style::default()
                };

                Row::new(cells).style(style)
            });

            let widths = [
                    Constraint::Percentage(25),
                    Constraint::Percentage(35),
                    Constraint::Percentage(15),
                    Constraint::Percentage(25),
                ];

            let table = Table::new(rows, widths)
                .header(header)
                .block(block)
                .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            // If we have a selected job, use a stateful table to highlight it
            if self.selected_job_index.is_some() {
                let mut state = TableState::default();
                state.select(self.selected_job_index);
                frame.render_stateful_widget(table, area, &mut state);
            } else {
                frame.render_widget(table, area);
            }
        }

        Ok(())
    }
}
