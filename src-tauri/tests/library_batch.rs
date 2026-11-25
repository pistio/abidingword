//! v0.24 batch import: several files or a folder, read one at a time,
//! reviewed and corrected, added with a shared collection and tags;
//! duplicates, problems, cancelling, a changed file, and continuing after a
//! crash. All material is invented.

use std::fs;
use std::path::Path;

use abiding_word_lib::commands::library_batch::{import_next, read_next, scan};
use abiding_word_lib::db::import_jobs::{self, ItemEdit, Job};
use abiding_word_lib::db::library_manage::{self, ManageQuery};
use abiding_word_lib::filesystem::DataDirs;
use abiding_word_lib::state::{AppState, Core};

fn write(dir: &Path, name: &str, text: &str) {
    let path = dir.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read_all(core: &Core, job: &Job) -> Job {
    let mut current = job.clone();
    let mut steps = 0;
    while current.counts.waiting > 0 {
        current = read_next(core, &job.id).unwrap();
        steps += 1;
        assert!(steps <= job.counts.total, "one file per step");
    }
    current
}

fn add_all(core: &Core, job: &Job) -> Job {
    let mut current = job.clone();
    for _ in 0..=job.counts.total {
        current = import_next(core, &job.id).unwrap();
        if current.state == "finished" || current.state == "cancelled" {
            break;
        }
    }
    current
}

fn titles(core: &Core) -> Vec<String> {
    let mut list: Vec<String> = core
        .db
        .with_conn(|conn| library_manage::list(conn, &ManageQuery::default()))
        .unwrap()
        .items
        .into_iter()
        .map(|s| s.title)
        .collect();
    list.sort();
    list
}

#[test]
fn a_folder_is_read_reviewed_and_added_with_a_shared_collection_and_tags() {
    let temp = tempfile::tempdir().unwrap();
    let state = AppState::start(DataDirs::ensure(temp.path().join("data")), None, None);
    let core = state.core().unwrap();
    let input = tempfile::tempdir().unwrap();
    let dir = input.path();
    write(
        dir,
        "Romans Commentary.md",
        "# Romans\n\nJustified by faith (Romans 5:1).",
    );
    write(
        dir,
        "sermons/Grace.md",
        "# Grace\n\nBy grace you have been saved.",
    );
    write(
        dir,
        "copy-of-grace.md",
        "# Grace\n\nBy grace you have been saved.",
    );
    write(dir, "empty.md", "");
    write(dir, "picture.png", "not a source");

    let job = scan(core, &[dir.to_string_lossy().into_owned()], true).unwrap();
    assert_eq!(job.counts.total, 4);
    assert_eq!(job.unsupported_count, 1);
    assert_eq!(job.counts.waiting, 4);
    // Nothing is added before the review.
    assert!(titles(core).is_empty());

    let job = read_all(core, &job);
    let by_name = |name: &str| {
        job.items
            .iter()
            .find(|i| i.file_name == name)
            .unwrap()
            .clone()
    };
    let romans = by_name("Romans Commentary.md");
    assert_eq!(romans.status, "ready");
    // A category only from evidence: the word in the file's name.
    assert_eq!(romans.source_type.as_deref(), Some("commentary"));
    assert_eq!(romans.type_evidence.as_deref(), Some("fileName"));
    let grace = by_name("Grace.md");
    assert_eq!(
        (grace.source_type.as_deref(), grace.type_evidence.as_deref()),
        (Some("sermon"), Some("folder"))
    );
    // The same content twice: the later one is a duplicate, not chosen.
    let (first, second) = (by_name("copy-of-grace.md"), grace.clone());
    let dup = [first, second]
        .into_iter()
        .find(|i| i.status == "duplicate")
        .unwrap();
    assert!(!dup.chosen);
    assert_eq!(by_name("empty.md").status, "problem");
    assert_eq!(job.counts.to_add, 2);

    // Corrections, and the shared collection and tags.
    core.db
        .with_conn(|conn| {
            import_jobs::edit_item(
                conn,
                &job.id,
                &ItemEdit {
                    id: romans.id.clone(),
                    title: Some("Romans: A Commentary".into()),
                    author: Some("A. Writer".into()),
                    language: None,
                    source_type: None,
                    chosen: None,
                },
            )?;
            let collection = library_manage::create_collection(conn, "New books", None)?;
            import_jobs::set_shared(
                conn,
                &job.id,
                Some(&collection.id),
                &["Imported".into(), " ".into()],
            )
        })
        .unwrap();

    let done = add_all(core, &job);
    assert_eq!(done.state, "finished");
    assert_eq!(done.counts.imported, 2);
    assert_eq!(titles(core), ["Grace", "Romans: A Commentary"]);
    let collections = core.db.with_conn(library_manage::collections).unwrap();
    assert_eq!(collections[0].count, 2);
    let tags = core.db.with_conn(library_manage::tags).unwrap();
    assert_eq!(
        tags.iter()
            .map(|t| (t.name.as_str(), t.count))
            .collect::<Vec<_>>(),
        [("Imported", 2)]
    );
    // The originals are untouched.
    assert!(dir.join("Romans Commentary.md").exists());
}

#[test]
fn cancelling_stops_between_files_and_keeps_what_was_added() {
    let temp = tempfile::tempdir().unwrap();
    let state = AppState::start(DataDirs::ensure(temp.path().join("data")), None, None);
    let core = state.core().unwrap();
    let input = tempfile::tempdir().unwrap();
    for n in 1..=3 {
        write(
            input.path(),
            &format!("note-{n}.md"),
            &format!("# Note {n}\n\nText {n}."),
        );
    }
    let job = scan(core, &[input.path().to_string_lossy().into_owned()], false).unwrap();
    let job = read_all(core, &job);
    let after_one = import_next(core, &job.id).unwrap();
    assert_eq!(after_one.state, "importing");
    assert_eq!(after_one.counts.imported, 1);
    core.db
        .with_conn(|conn| import_jobs::set_state(conn, &job.id, "cancelled"))
        .unwrap();
    let after = import_next(core, &job.id).unwrap();
    assert_eq!(after.counts.imported, 1);
    assert_eq!(titles(core), ["Note 1"]);
    // An ended job cannot be changed.
    let edit = core
        .db
        .with_conn(|conn| import_jobs::retry(conn, &job.id, &[job.items[1].id.clone()]));
    assert!(edit.is_err());
}

#[test]
fn after_a_crash_the_job_is_interrupted_and_continues_without_adding_twice() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("data");
    let input = tempfile::tempdir().unwrap();
    for n in 1..=3 {
        write(
            input.path(),
            &format!("part-{n}.md"),
            &format!("# Part {n}\n\nWords {n}."),
        );
    }
    let job_id = {
        let state = AppState::start(DataDirs::ensure(&root), None, None);
        let core = state.core().unwrap();
        let job = scan(core, &[input.path().to_string_lossy().into_owned()], false).unwrap();
        let job = read_all(core, &job);
        import_next(core, &job.id).unwrap();
        // The second file is added, then the app stops before its item is marked.
        let item = job.items[1].clone();
        let staged = abiding_word_lib::library::import::stage(Path::new(&item.path)).unwrap();
        abiding_word_lib::commands::library::add_staged(
            core,
            &staged,
            &abiding_word_lib::library::CommitSourceImport {
                preview_id: staged.id.clone(),
                title: "Part 2".into(),
                ..Default::default()
            },
        )
        .unwrap();
        job.id
    };
    let state = AppState::start(DataDirs::ensure(&root), None, None);
    let core = state.core().unwrap();
    let open = core.db.with_conn(import_jobs::open_jobs).unwrap();
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].state, "interrupted");
    assert_eq!(open[0].counts.imported, 1);
    let done = add_all(core, &open[0]);
    assert_eq!(done.id, job_id);
    assert_eq!(done.state, "finished");
    assert_eq!(done.counts.imported, 3);
    assert_eq!(titles(core), ["Part 1", "Part 2", "Part 3"]);
}

#[test]
fn a_file_changed_after_its_check_is_not_added_and_can_be_read_again() {
    let temp = tempfile::tempdir().unwrap();
    let state = AppState::start(DataDirs::ensure(temp.path().join("data")), None, None);
    let core = state.core().unwrap();
    let input = tempfile::tempdir().unwrap();
    write(input.path(), "a.md", "# A\n\nFirst words.");
    let job = scan(
        core,
        &[input.path().join("a.md").to_string_lossy().into_owned()],
        false,
    )
    .unwrap();
    let job = read_all(core, &job);
    write(input.path(), "a.md", "# A\n\nOther words.");
    let after = import_next(core, &job.id).unwrap();
    assert_eq!(after.items[0].status, "failed");
    assert_eq!(
        after.items[0].problem_code.as_deref(),
        Some("batch.changed")
    );
    assert!(titles(core).is_empty());
    core.db
        .with_conn(|conn| import_jobs::retry(conn, &job.id, &[job.items[0].id.clone()]))
        .unwrap();
    let job = read_all(
        core,
        &core.db.with_conn(|c| import_jobs::get(c, &job.id)).unwrap(),
    );
    let done = add_all(core, &job);
    assert_eq!(done.counts.imported, 1);
    assert_eq!(titles(core), ["A"]);
}

#[test]
fn a_job_is_refused_when_nothing_readable_is_found() {
    let temp = tempfile::tempdir().unwrap();
    let state = AppState::start(DataDirs::ensure(temp.path().join("data")), None, None);
    let core = state.core().unwrap();
    let input = tempfile::tempdir().unwrap();
    write(input.path(), "x.exe", "no");
    let refused = scan(core, &[input.path().to_string_lossy().into_owned()], true);
    assert!(refused.is_err());
}
