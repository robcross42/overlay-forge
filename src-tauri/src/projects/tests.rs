use super::{domain::*, repository::ProjectRepository};
use crate::db::HostDatabase;

fn draft(title: &str) -> ProjectDraft {
    ProjectDraft {
        title: title.into(),
        description: "Synthetic acceptance record".into(),
        stage: ProjectStage::InProgress,
        next_step: "Review this record".into(),
        waiting_on_rob: "Check the synthetic preview".into(),
        links: vec![],
    }
}

#[test]
fn project_lifecycle_persists_and_never_touches_linked_files() {
    let root = std::env::temp_dir().join(format!(
        "overlay-forge-project-check-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&root).unwrap();
    let linked = root.join("linked.txt");
    std::fs::write(&linked, b"unchanged linked content").unwrap();
    let path = root.join("host.sqlite3");
    let id;
    {
        let database = HostDatabase::new(path.clone()).unwrap();
        let repository = ProjectRepository::new(&database);
        let mut input = draft("Acceptance entry");
        input.links.push(ProjectLink {
            label: "Document".into(),
            target: linked.to_string_lossy().into(),
        });
        id = repository.save(None, input.clone()).unwrap();
        input.title = "Edited acceptance entry".into();
        repository.save(Some(&id), input).unwrap();
        assert!(repository
            .delete(&id, "Edited acceptance entry")
            .unwrap_err()
            .contains("Archive"));
        // Even direct SQL must enforce the archive prerequisite.
        assert!(database
            .connection()
            .unwrap()
            .execute("DELETE FROM obj_host_project WHERE id=?1", [&id])
            .is_err());
    }
    {
        let database = HostDatabase::new(path).unwrap();
        let repository = ProjectRepository::new(&database);
        let normal = ProjectQuery {
            search: "Edited acceptance entry".into(),
            ..Default::default()
        };
        let restored = repository.list(&normal).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].draft.stage, ProjectStage::InProgress);
        assert_eq!(restored[0].draft.links[0].target, linked.to_string_lossy());
        let waiting = ProjectQuery {
            waiting_only: true,
            ..normal
        };
        assert_eq!(repository.list(&waiting).unwrap().len(), 1);
        repository.set_archived(&id, true).unwrap();
        assert!(repository.list(&waiting).unwrap().is_empty());
        let archived = ProjectQuery {
            archived: true,
            ..waiting
        };
        assert_eq!(repository.list(&archived).unwrap().len(), 1);
        assert_eq!(
            serde_json::to_value(&repository.list(&archived).unwrap()[0].draft).unwrap(),
            serde_json::to_value(&restored[0].draft).unwrap()
        );
        repository.set_archived(&id, false).unwrap();
        assert_eq!(
            repository
                .list(&ProjectQuery {
                    search: "Edited acceptance entry".into(),
                    ..Default::default()
                })
                .unwrap()
                .len(),
            1
        );
        repository.set_archived(&id, true).unwrap();
        assert!(repository.delete(&id, "wrong title").is_err());
        repository.delete(&id, "Edited acceptance entry").unwrap();
        assert!(repository.list(&archived).unwrap().is_empty());
        assert_eq!(std::fs::read(&linked).unwrap(), b"unchanged linked content");
    }
    // Only this test's UUID-scoped temporary fixture is removed.
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_seeds_preserve_edits_archives_and_deletions_across_initialization() {
    let database = HostDatabase::new(std::path::PathBuf::from(":memory:")).unwrap();
    let repository = ProjectRepository::new(&database);
    let initial = repository.list(&ProjectQuery::default()).unwrap();
    assert_eq!(initial.len(), super::seeds::catalog().len());
    assert_eq!(
        initial
            .iter()
            .filter(|item| item.draft.title.contains("Retirement"))
            .count(),
        1
    );
    let seed = &initial[0];
    repository
        .save(Some(&seed.id), draft("Owner's edited seed"))
        .unwrap();
    repository.set_archived(&seed.id, true).unwrap();
    ProjectRepository::initialize(&database.connection().unwrap()).unwrap();
    repository.seed_once().unwrap();
    assert_eq!(
        repository.list(&ProjectQuery::default()).unwrap().len(),
        initial.len() - 1
    );
    let archived = repository
        .list(&ProjectQuery {
            archived: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(archived[0].draft.title, "Owner's edited seed");
    repository.delete(&seed.id, "Owner's edited seed").unwrap();
    repository.seed_once().unwrap();
    assert_eq!(
        repository.list(&ProjectQuery::default()).unwrap().len(),
        initial.len() - 1
    );
    assert!(repository
        .list(&ProjectQuery {
            archived: true,
            ..Default::default()
        })
        .unwrap()
        .is_empty());
}

#[test]
fn project_filters_and_input_validation() {
    let database = HostDatabase::new(std::path::PathBuf::from(":memory:")).unwrap();
    let repository = ProjectRepository::new(&database);
    let query = ProjectQuery {
        stage: Some(ProjectStage::ReadyForReview),
        waiting_only: true,
        ..Default::default()
    };
    let matches = repository.list(&query).unwrap();
    assert_eq!(matches.len(), 1);
    assert!(matches[0].draft.waiting_on_rob.contains("Kibofa Valley"));
    let mut input = draft(" ");
    assert!(input.clone().validate().is_err());
    input.title = "Valid title".into();
    for target in [
        "javascript:alert(1)",
        "relative/path",
        "file:///C:/test",
        "https://user:secret@example.com",
        "C:\\te\nst",
    ] {
        input.links = vec![ProjectLink {
            label: "Reference".into(),
            target: target.into(),
        }];
        assert!(
            input.clone().validate().is_err(),
            "unexpectedly accepted {target}"
        );
    }
}
