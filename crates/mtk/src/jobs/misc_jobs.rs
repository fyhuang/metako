use serde::{de::DeserializeOwned, Serialize};

use super::JobSpec;

use crate::FsEntry;
use crate::catalog::generated_notes;

pub struct UpdateGeneratedNotesJobSpec<CheckNeededFn, GenerateFn> {
    job_type: String,
    group_name: String,
    check_needed_fn: CheckNeededFn,
    generate_fn: GenerateFn,
}

impl <CheckNeededFn, GenerateFn> UpdateGeneratedNotesJobSpec<CheckNeededFn, GenerateFn> {
    pub fn new(job_type: &str, group_name: &str, check_needed_fn: CheckNeededFn, generate_fn: GenerateFn) -> Self {
        Self {
            job_type: job_type.to_string(),
            group_name: group_name.to_string(),
            check_needed_fn,
            generate_fn,
        }
    }
}

impl <CheckNeededFn, GeneratedT, GenerateFn> JobSpec for UpdateGeneratedNotesJobSpec<CheckNeededFn, GenerateFn> where 
    CheckNeededFn: Fn(&crate::Entry) -> bool + 'static,
    GeneratedT: Serialize + DeserializeOwned,
    GenerateFn: Fn(&FsEntry) -> Result<GeneratedT, Box<dyn std::error::Error>> + Copy + 'static,
{
    fn job_type(&self) -> &str {
        &self.job_type
    }

    fn create_job(&self, stash: &crate::Vault, entry: &crate::Entry) -> Result<Option<Box<crate::jobs::JobFn>>, Box<dyn std::error::Error>> {
        if !(self.check_needed_fn)(entry) || !generated_notes::needs_update(entry, &self.group_name) {
            return Ok(None);
        }
        let group_name = self.group_name.clone();
        let entry_id = entry.db.id;
        let fs_entry = entry.fs.clone();
        let generate_fn = self.generate_fn;
        let mut catalog = stash.open_catalog()?;
        Ok(Some(Box::new(move || {
            println!("Updating generated \"{}\" notes for {}", &group_name, fs_entry.repo_path);
            let generated = (generate_fn)(&fs_entry)?;
            generated_notes::update(&mut catalog, entry_id, &group_name, &generated);
            Ok(())
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    use crate::{testing, Entry, RepoPathBuf};

    #[test]
    fn test_update_generated_notes_job_spec() {
        let file_root = testing::testdata_path("mixed");
        let (_tempdir, stash) = testing::tempdir_vault(&file_root).expect("tempdir_stash");

        let mut catalog = stash.open_catalog().expect("open catalog");
        let fs_entry = stash.new_file_tree().get_fs_entry(&RepoPathBuf::from("Videos/berlin_wall.mp4")).expect("get_fs_entry");
        let entry_id = catalog.get_or_create(&fs_entry).id;

        let generate_fn = |_: &FsEntry| {
            Ok(json!({
                "string_key": "value1",
                "u64_key": 42,
                "float_key": 3.14,
            }))
        };

        let job_spec = UpdateGeneratedNotesJobSpec::new(
            "update_generated_notes",
            "group1",
            |_: &Entry| true,
            generate_fn,
        );

        let db_entry = catalog.get_by_id(entry_id).expect("get entry");
        let entry = Entry {
            fs: fs_entry.clone(),
            db: db_entry,
        };

        assert!(job_spec.create_job(&stash, &entry).unwrap().is_some());

        let job = job_spec.create_job(&stash, &entry).expect("create job").expect("job needed");
        job().expect("run job");

        let updated_entry = stash.open_catalog().expect("open catalog").get_by_id(entry_id).expect("get entry");
        let generated_map = updated_entry.notes_generated.as_object().expect("as_object");
        assert!(generated_map.contains_key("group1::__last_update"));  // the actual value is not deterministic
        assert_eq!(&generated_map["group1::string_key"], &json!("value1"));
        assert_eq!(&generated_map["group1::u64_key"], &json!(42));
        assert_eq!(&generated_map["group1::float_key"], &json!(3.14));

        let updated_entry = Entry {
            fs: fs_entry,
            db: updated_entry,
        };
        assert_eq!(false, job_spec.create_job(&stash, &updated_entry).unwrap().is_some());
    }
}
