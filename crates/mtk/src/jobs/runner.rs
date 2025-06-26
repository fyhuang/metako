use super::registry::JobRegistry;

pub struct JobRunner<'a> {
    stash: &'a crate::Vault,
    registry: JobRegistry,

    catalog: crate::catalog::sqlite_catalog::Catalog,
    file_tree: crate::file_tree::FileTree,
}

impl<'a> JobRunner<'a> {
    pub fn new(stash: &'a crate::Vault, registry: JobRegistry) -> Self {
        Self {
            stash,
            registry,

            catalog: stash.open_catalog().expect("open_catalog"),
            file_tree: stash.new_file_tree(),
        }
    }

    pub fn run_one(&self, entry_id: i64, job_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        let Some(job_spec) = self.registry.get(job_type) else {
            return Err(format!("Unknown job type: {}", job_type).into());
        };

        let Some(db_entry) = self.catalog.get_by_id(entry_id) else {
            return Err(format!("No entry with ID: {}", entry_id).into());
        };

        let fs_entry = self.file_tree.get_fs_entry(&db_entry.repo_path)?;
        let entry = crate::Entry {
            fs: fs_entry,
            db: db_entry,
        };

        let job = job_spec.create_job(self.stash, &entry)?;
        match job {
            Some(job_fn) => job_fn(),
            None => Ok(()),
        }
    }

    pub fn run_until_empty(&self) {
        //let queue = self.stash.open_job_queue();
        loop {
            // Get the next job from the queue and run it
            todo!();
        }
    }

    pub fn run_loop(self) {
        loop {
            // Get the next job from the queue and run it
            todo!();
        }
    }
}
