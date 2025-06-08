use std::path::{Path, PathBuf};

use super::generated_file::{GeneratedFile, GeneratedFileType};

const GENERATED_DIR: &str = "generated";

pub struct GeneratedTree {
    base_path: PathBuf,
}

impl GeneratedTree {
    pub fn new(base_path: &Path) -> GeneratedTree {
        GeneratedTree { base_path: base_path.to_path_buf() }
    }

    fn parent_dir(&self, entry_id: i64) -> PathBuf {
        let entry_id_padded = format!("{:09}", entry_id);

        assert_eq!(entry_id_padded.len() % 3, 0);

        let mut parent_dir = self.base_path.join(GENERATED_DIR);
        for index in 0..(entry_id_padded.len() / 3 - 1) {
            let digit_group = &entry_id_padded[index * 3..(index + 1) * 3];
            parent_dir = parent_dir.join(&digit_group);
        }

        parent_dir
    }

    pub fn path_to_generated_file(&self, gfile: &GeneratedFile) -> PathBuf {
        let parent_dir = self.parent_dir(gfile.entry_id);
        if !parent_dir.exists() {
            std::fs::create_dir_all(&parent_dir).expect("create_dir_all");
        }

        let filename = format!(
            "{}__{}.{}.{}",
            gfile.entry_id,
            gfile.file_type.to_two_letter_code(),
            gfile.metadata,
            gfile.extension,
        );
        parent_dir.join(filename)
    }

    pub fn is_generated(&self, path: &Path) -> bool {
        path.starts_with(&self.base_path.join(GENERATED_DIR))
    }

    pub fn query_generated_files(
        &self,
        entry_id: i64,
        file_type: GeneratedFileType,
    ) -> Vec<GeneratedFile> {
        let pattern = format!(
            "{}/{}__{}.*.*",
            self.parent_dir(entry_id).to_str().expect("to_str"),
            entry_id,
            file_type.to_two_letter_code()
        );

        let mut result = Vec::new();
        for entry in glob::glob(&pattern).expect("glob") {
            let path = entry.expect("entry");

            // Parse the filename using a regex
            let filename = path
                .file_name()
                .expect("file_name")
                .to_str()
                .expect("to_str");
            let re = regex::Regex::new(r"(\d+)__([A-Z0-9]{2})\.([^.]*)\.(.*)").expect("regex");
            let caps = re.captures(filename).expect("captures");
            let file_type =
                GeneratedFileType::from_two_letter_code(caps.get(2).expect("file_type").as_str())
                    .expect("from_two_letter_code");
            let metadata = caps.get(3).expect("metadata").as_str();
            let extension = caps.get(4).expect("extension").as_str();

            result.push(GeneratedFile {
                entry_id,
                file_type,
                metadata: metadata.to_string(),
                extension: extension.to_string(),
            });
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_generated_file() -> std::io::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path();

        let gen_tree = GeneratedTree {
            base_path: file_root.to_path_buf(),
        };

        let gfile = GeneratedFile {
            entry_id: 123,
            file_type: GeneratedFileType::Subtitle,
            metadata: "en_US".to_string(),
            extension: "vtt".to_string(),
        };
        let generated_file_path = gen_tree.path_to_generated_file(&gfile);
        assert!(generated_file_path.starts_with(file_root));
        assert!(generated_file_path.ends_with("123__ST.en_US.vtt"));

        // Ensure the directory is created
        let expected_parent_dir = file_root.join("generated/000/000");
        assert!(expected_parent_dir.is_dir());

        Ok(())
    }

    #[test]
    fn test_path_to_generated_file_large_entry_id() -> std::io::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path();

        let gen_tree = GeneratedTree {
            base_path: file_root.to_path_buf(),
        };

        let gfile = GeneratedFile {
            entry_id: 123456789,
            file_type: GeneratedFileType::Subtitle,
            metadata: "en_US".to_string(),
            extension: "vtt".to_string(),
        };
        let generated_file_path = gen_tree.path_to_generated_file(&gfile);
        assert!(generated_file_path.starts_with(file_root));
        assert!(generated_file_path.ends_with("123456789__ST.en_US.vtt"));

        // Ensure the directory is created
        let expected_parent_dir = file_root.join("generated/123/456");
        assert!(expected_parent_dir.is_dir());

        Ok(())
    }

    #[test]
    fn test_path_to_generated_file_zero_entry_id() -> std::io::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path();

        let gen_tree = GeneratedTree {
            base_path: file_root.to_path_buf(),
        };

        let gfile = GeneratedFile {
            entry_id: 0,
            file_type: GeneratedFileType::Subtitle,
            metadata: "en_US".to_string(),
            extension: "vtt".to_string(),
        };
        let generated_file_path = gen_tree.path_to_generated_file(&gfile);
        assert!(generated_file_path.starts_with(file_root));
        assert!(generated_file_path.ends_with("0__ST.en_US.vtt"));

        // Ensure the directory is created
        let expected_parent_dir = file_root.join("generated/000/000");
        assert!(expected_parent_dir.is_dir());

        Ok(())
    }

    #[test]
    fn test_path_to_generated_file_empty_metadata() -> std::io::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path();

        let gen_tree = GeneratedTree {
            base_path: file_root.to_path_buf(),
        };

        let gfile = GeneratedFile {
            entry_id: 123,
            file_type: GeneratedFileType::Preview,
            metadata: "".to_string(),
            extension: "jpg".to_string(),
        };
        let generated_file_path = gen_tree.path_to_generated_file(&gfile);
        assert!(generated_file_path.starts_with(file_root));
        assert!(generated_file_path.ends_with("123__PR..jpg"));

        Ok(())
    }

    #[test]
    fn test_query_generated_files() -> std::io::Result<()> {
        let tempdir = tempfile::tempdir()?;
        let file_root = tempdir.path();

        let gen_tree = GeneratedTree {
            base_path: file_root.to_path_buf(),
        };

        // Create 2 subtitle files and one AltFormat file
        std::fs::write(
            gen_tree.path_to_generated_file(&GeneratedFile {
                entry_id: 123,
                file_type: GeneratedFileType::Subtitle,
                metadata: "en_US".to_string(),
                extension: "vtt".to_string(),
            }),
            "test",
        )?;

        std::fs::write(
            gen_tree.path_to_generated_file(&GeneratedFile {
                entry_id: 123,
                file_type: GeneratedFileType::Subtitle,
                metadata: "zh_CN".to_string(),
                extension: "vtt".to_string(),
            }),
            "test",
        )?;

        std::fs::write(
            gen_tree.path_to_generated_file(&GeneratedFile {
                entry_id: 123,
                file_type: GeneratedFileType::AltFormat,
                metadata: "1080p".to_string(),
                extension: "mp4".to_string(),
            }),
            "test",
        )?;

        // Create one unrelated file (with a different entry ID)
        std::fs::write(
            gen_tree.path_to_generated_file(&GeneratedFile {
                entry_id: 1234,
                file_type: GeneratedFileType::AltFormat,
                metadata: "720p".to_string(),
                extension: "mp4".to_string(),
            }),
            "test",
        )?;

        let files = gen_tree.query_generated_files(123, GeneratedFileType::Subtitle);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].entry_id, 123);
        assert_eq!(files[0].file_type, GeneratedFileType::Subtitle);
        assert_eq!(files[0].extension, "vtt");
        assert_eq!(files[1].entry_id, 123);
        assert_eq!(files[1].file_type, GeneratedFileType::Subtitle);
        assert_eq!(files[1].extension, "vtt");

        // The returned order isn't guaranteed
        let metadata_set = files
            .iter()
            .map(|f| f.metadata.clone())
            .collect::<std::collections::HashSet<String>>();
        assert!(metadata_set.contains("en_US"));
        assert!(metadata_set.contains("zh_CN"));

        // Query for AltFormat files
        let af_files = gen_tree.query_generated_files(123, GeneratedFileType::AltFormat);
        assert_eq!(af_files.len(), 1); // shouldn't include the 720p file for different entry
        assert_eq!(af_files[0].entry_id, 123);
        assert_eq!(af_files[0].file_type, GeneratedFileType::AltFormat);
        assert_eq!(af_files[0].metadata, "1080p");
        assert_eq!(af_files[0].extension, "mp4");

        Ok(())
    }
}
