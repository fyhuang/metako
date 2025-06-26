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
pub fn default_registry(stash: &crate::Vault) -> JobRegistry {
    /*let catalog = stash.open_catalog().expect("open_catalog");
    let gen_tree = stash.new_generated_tree();
    let repo_path = catalog.get_by_id(entry_id).expect("get_by_id").repo_path;
    let file_path = stash.new_file_tree().repo_to_full_path(&repo_path);
    match job_type {
        "preview" => {
            Box::new(crate::preview::PreviewJob {
                file_path: file_path,
                entry_id: entry_id,
                gen_tree: gen_tree,
            })
        },
        "transcode" => Box::new(crate::video::TranscodeJob {
            video_path: file_path,
            entry_id: entry_id,
            gen_tree: gen_tree,
        }),
        "subtitles" => Box::new(crate::video::ConvertSubtitlesJob {
            video_path: file_path,
            entry_id: entry_id,
            gen_tree: gen_tree,
        }),
        "dummy" => Box::new(file_path.to_str().expect("to_str").to_string()),
        _ => panic!("Unknown job type: {}", job_type),
    }*/
    let mut registry = JobRegistry::new();
    registry.register(Box::new(super::media_jobs::PreviewJobSpec{}));
    registry.register(Box::new(super::media_jobs::generate_video_info_job_spec()));
    registry
}

#[cfg(test)]
mod tests {
    use crate::testing;

    use super::*;

    #[test]
    fn test_default_registry() {
        let file_root = testing::testdata_path("mixed");
        let (_tempdir, stash) = testing::tempdir_vault(&file_root).expect("tempdir_stash");

        let registry = default_registry(&stash);
        assert!(registry.get("preview").is_some());
        assert!(registry.get("video_info").is_some());
    }
}
