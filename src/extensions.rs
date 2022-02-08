use std::path::{ Component, Path, PathBuf };

pub trait PathExt {
    fn relativize(&self, parent: impl AsRef<Path>) -> PathBuf;
}

impl PathExt for Path {
    fn relativize(&self, parent: impl AsRef<Path>) -> PathBuf {
        let mut result = PathBuf::new();
        let mut parent_iter = parent.as_ref().components();
        let mut child_iter = self.components();
        loop {
            let parent_comp = parent_iter.next();
            let child_comp = child_iter.next();
            if let Some(parent_comp) = parent_comp {
                if let Some(child_comp) = child_comp {
                    if parent_comp != child_comp {
                        break;
                    }
                }
            } else if let Some(child_comp) = child_comp {
                result.push(child_comp);
                for comp in child_iter {
                    result.push(comp);
                }
                break;
            }
        }
        result
    }
}

impl PathExt for PathBuf {
    fn relativize(&self, parent: impl AsRef<Path>) -> PathBuf {
        self.as_path().relativize(parent)
    }
}


#[test]
fn test_relativize() {
    let parent = PathBuf::from("parent/path");
    let child = PathBuf::from("parent/path/child/path.txt");
    let relativized = child.relativize(parent);
    assert_eq!(PathBuf::from("child/path.txt").as_path(), &relativized);
    let parent = PathBuf::from("parent/pathzzzz");
    let child = PathBuf::from("parent/path/child/path.txt");
    let relativized = child.relativize(parent);
    assert_eq!(PathBuf::from("").as_path(), &relativized);
}