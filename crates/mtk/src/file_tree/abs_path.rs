use std::path::{Path, PathBuf, Component};

pub fn to_abs_path(input_path: &Path) -> PathBuf {
    let input_components: Vec<Component> = input_path.components().collect();
    if input_components.len() == 0 {
        return input_path.to_path_buf();
    }

    let mut output_components: Vec<Component> = Vec::new();

    // If input doesn't start with rootdir, prepend components from cwd.
    let input_has_root = match input_components[0] {
        Component::Prefix(_) => true,
        Component::RootDir => true,
        _ => false,
    };

    let current_dir = std::env::current_dir().unwrap();
    if !input_has_root {
        let mut cwd_components: Vec<Component> = current_dir.components().collect();
        output_components.append(&mut cwd_components);
    }

    // Treating output_components as a stack, walk the input components
    for c in input_components {
        match c {
            Component::CurDir => continue,
            // TODO(fyhuang): handle not being able to advance past root
            Component::ParentDir => { output_components.pop(); }
            _ => output_components.push(c),
        }
    }

    let mut output_path = PathBuf::new();
    for c in output_components {
        output_path.push(c);
    }
    output_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_abs_path() {
        let cwd = std::env::current_dir().unwrap();

        assert_eq!(to_abs_path(Path::new("/my/abs/path")), PathBuf::from("/my/abs/path"));
        assert_eq!(to_abs_path(Path::new("/my/abs/path/")), PathBuf::from("/my/abs/path"));
        assert_eq!(to_abs_path(Path::new("my/rel/path")), cwd.join("my/rel/path"));
        assert_eq!(to_abs_path(Path::new("my/../rel/path")), cwd.join("rel/path"));
        assert_eq!(to_abs_path(Path::new("./my/../rel/.././path/..")), cwd);
    }

}
