use std::fs;

use importmap::ImportMap;
use tempfile::tempdir;

#[cfg(unix)]
use std::os::unix::fs::symlink;

const INCLUDED: &[&str] = &[
    "style.css",
    "app.mjs",
    "modules/app.js",
    "modules/style.css",
];
const EXCLUDED: &[&str] = &[
    "service-worker.js",
    "image.png",
    "modules/_private.js",
    "modules/app.development.js",
    "modules/app.dev.mjs",
    "modules/app.test.css",
    "tests/app.js",
    "modules/tests/style.css",
];

#[test]
fn filesystem_scan_preserves_selection_and_refreshes_content_hashes() {
    let dir = tempdir().unwrap();
    for path in INCLUDED.iter().chain(EXCLUDED) {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "first").unwrap();
    }
    let before = ImportMap::scan(dir.path(), "/assets/").unwrap();
    assert_eq!(before.len(), INCLUDED.len());
    for path in INCLUDED {
        assert!(before.contains_key(&format!("/assets/{path}")));
    }
    fs::write(dir.path().join("modules/app.js"), "second").unwrap();
    let after = ImportMap::scan(dir.path(), "/assets/").unwrap();
    assert_ne!(
        before["/assets/modules/app.js"],
        after["/assets/modules/app.js"]
    );
    assert_eq!(before["/assets/style.css"], after["/assets/style.css"]);
}

#[cfg(unix)]
#[test]
fn excluded_files_are_not_read_but_included_read_errors_propagate() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("missing-target");
    for path in EXCLUDED {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        symlink(&missing, path).unwrap();
    }
    assert!(ImportMap::scan(dir.path(), "").unwrap().is_empty());
    symlink(&missing, dir.path().join("modules/app.js")).unwrap();
    assert!(ImportMap::scan(dir.path(), "").is_err());
}
