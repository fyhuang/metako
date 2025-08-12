use std::collections::HashMap;

use super::JobSpec;

pub struct JobRegistry {
    specs: HashMap<String, Box<dyn JobSpec>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            specs: HashMap::new(),
        }
    }

    pub fn register(&mut self, spec: Box<dyn JobSpec>) {
        self.specs.insert(spec.job_type().to_string(), spec);
    }

    pub fn get(&self, job_type: &str) -> Option<&Box<dyn JobSpec>> {
        self.specs.get(job_type)
    }
}

// TODO: where to put this?
pub fn default_registry() -> JobRegistry {
    let mut registry = JobRegistry::new();
    registry.register(Box::new(super::media_jobs::PreviewJobSpec{}));
    registry.register(Box::new(super::media_jobs::generate_video_info_job_spec()));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry() {
        let registry = default_registry();
        assert!(registry.get("preview").is_some());
        assert!(registry.get("video_info").is_some());
    }
}
