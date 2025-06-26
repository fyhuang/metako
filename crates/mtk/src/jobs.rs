pub type JobFn = dyn FnOnce() -> Result<(), Box<dyn std::error::Error>>;

pub trait JobSpec {
    fn job_type(&self) -> &str;
    /// Returns Ok(Some(job)) if the job is needed, Ok(None) if not needed, or Err on error.
    fn create_job(
        &self,
        stash: &crate::Vault,
        entry: &crate::Entry,
    ) -> Result<Option<Box<JobFn>>, Box<dyn std::error::Error>>;
}

pub mod registry;
pub mod runner;

mod media_jobs;
pub use media_jobs::generate_video_info_job_spec;

mod misc_jobs;
