// TODO review this whole file
use aws_sdk_batch::{
    types::{JobDetail, JobStatus, JobSummary, JobQueueDetail},
    Client as BatchClient,
};
use aws_config::SdkConfig;
use std::sync::{Arc, Mutex};

/// Mock implementation of AWS Batch client for testing
pub struct MockBatchClient {
    job_queues: Arc<Mutex<Vec<JobQueueDetail>>>,
    running_jobs: Arc<Mutex<Vec<JobSummary>>>,
    submitted_jobs: Arc<Mutex<Vec<JobSummary>>>,
}

impl MockBatchClient {
    /// Create a new mock batch client with empty data
    pub fn new() -> Self {
        Self {
            job_queues: Arc::new(Mutex::new(Vec::new())),
            running_jobs: Arc::new(Mutex::new(Vec::new())),
            submitted_jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a job queue to the mock client
    pub fn add_job_queue(&self, queue_name: &str) {
        let queue = JobQueueDetail::builder()
            .job_queue_name(queue_name)
            .build();

        let mut queues = self.job_queues.lock().unwrap();
        queues.push(queue);
    }

    /// Add a running job to the mock client
    pub fn add_running_job(&self, job_id: &str, job_name: &str, queue_name: &str) {
        let job = JobSummary::builder()
            .job_id(job_id)
            .job_name(job_name)
            // .job_queue(queue_name) - removed as method doesn't exist
            .status(JobStatus::Running)
            .created_at(chrono::Utc::now().timestamp())
            .build();

        let mut jobs = self.running_jobs.lock().unwrap();
        jobs.push(job);
    }

    /// Add a submitted job to the mock client
    pub fn add_submitted_job(&self, job_id: &str, job_name: &str, queue_name: &str) {
        let job = JobSummary::builder()
            .job_id(job_id)
            .job_name(job_name)
            // .job_queue(queue_name) - removed as method doesn't exist
            .status(JobStatus::Submitted)
            .created_at(chrono::Utc::now().timestamp())
            .build();

        let mut jobs = self.submitted_jobs.lock().unwrap();
        jobs.push(job);
    }

    /// Create a mock AWS config that can be used to create a mock client
    pub fn mock_config() -> SdkConfig {
        // Create a simple config for testing
        SdkConfig::builder()
            .build()
    }
}

/// Factory function to create a mock BatchClient
pub fn create_mock_batch_client() -> BatchClient {
    // This is a placeholder - in a real implementation, we would need to
    // create a proper mock that implements the same traits as BatchClient
    // For now, we'll just return a real client with a mock config
    // that won't actually connect to AWS
    BatchClient::new(&MockBatchClient::mock_config())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::batch::{Batch, JobInfo};
    use crate::components::Component;

    #[tokio::test]
    async fn test_batch_component_with_mock() {
        // Create a mock client
        let mock_client = create_mock_batch_client();

        // Create a batch component with the mock client
        let mut batch = Batch::new();
        batch.register_aws_config_handler(MockBatchClient::mock_config()).unwrap();

        // TODO: Add assertions to verify the batch component behavior
        // This would require modifying the Batch component to accept a mock client
        // or to have a way to inject mock responses
    }
}
