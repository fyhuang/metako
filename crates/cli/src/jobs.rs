use std::path::PathBuf;

use mtk::RepoPathBuf;

fn run_job_on_entries(stash: &mtk::Vault, job_type: &str, entry_iter: impl Iterator<Item = mtk::Entry>) {
    let runner = mtk::jobs::runner::JobRunner::new(stash, mtk::jobs::registry::default_registry());

    for entry in entry_iter {
        let run_result = runner.run_one(entry.db.id, job_type);
        match run_result {
            Ok(()) => {},
            Err(e) => {
                eprintln!("Error running job {} for {:?}: {:?}", job_type, entry.fs.file_path, e);
            }
        }
    }
}

fn run_job_on_files(stash: &mtk::Vault, job_type: &str, files: &Vec<PathBuf>) {
    let file_tree = stash.new_file_tree();
    let mut catalog = stash.open_catalog().expect("open_catalog");

    let iter = files.iter().map(|file| {
        // TODO: don't use canonicalize
        let file_path = file.canonicalize().expect("canonicalize");
        let repo_path = file_tree.full_to_repo_path(&file_path).expect("full_to_repo_path");
        let fs_entry = file_tree.get_fs_entry(&repo_path).expect("get_fs_entry");
        // TODO: use scan instead
        let db_entry = catalog.get_or_create(&fs_entry);

        mtk::Entry {
            fs: fs_entry,
            db: db_entry,
        }
    });
    run_job_on_entries(stash, job_type, iter);
}

fn run_job_scan_media(stash: &mtk::Vault, job_type: &str, search_root: &RepoPathBuf) {
    let file_tree = stash.new_file_tree();
    let mut catalog = stash.open_catalog().expect("open_catalog");

    println!("Scanning for media files in {}", search_root);
    let listing = mtk::browse::list_recursive(&mut catalog, &file_tree, search_root).expect("list_recursive");

    println!("Running jobs on {} entries", listing.visible.len());
    run_job_on_entries(stash, job_type, listing.visible.into_iter().filter(|entry| entry.fs.file_type.is_file));
}

#[derive(clap::Args)]
pub struct RunJobsCommand {
    /// Scan the entire repo for media files
    #[arg(long)]
    scan_media: bool,

    job_name: String,
    in_files: Vec<PathBuf>,
}

impl RunJobsCommand {
    pub fn run(&self, stash: &mtk::Vault) {
        if self.scan_media {
            let search_root = if self.in_files.is_empty() {
                RepoPathBuf::from("")
            } else {
                let file_tree = stash.new_file_tree();
                let repo_path = file_tree.full_to_repo_path(&self.in_files[0]).expect("full_to_repo_path");
                repo_path
            };

            run_job_scan_media(stash, &self.job_name, &search_root);
        } else {
            if self.in_files.is_empty() {
                eprintln!("No files specified!");
                return;
            }
            run_job_on_files(stash, &self.job_name, &self.in_files);
        }
    }
}
