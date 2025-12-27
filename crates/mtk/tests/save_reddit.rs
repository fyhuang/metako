// Reddit integration tests

use url::Url;

use mtk::save;

#[test]
#[ignore]
fn test_save_reddit_image_post() {
    // Reddit post with single image (i.redd.it)
    let url = Url::parse("https://www.reddit.com/r/pics/comments/haucpf/ive_found_a_few_funny_memories_during_lockdown/").unwrap();
    let analysis = save::analyze(&url).expect("analyze");
    assert_eq!(analysis.targets.len(), 1);

    println!("Reddit image post analysis:");
    println!("  Title: {:?}", analysis.title);
    println!("  Targets: {} ({:?})", analysis.targets.len(), analysis.targets[0]);

    let temp_dir = tempfile::tempdir().unwrap();
    save::download(&analysis.targets[0], temp_dir.path()).expect("download");

    // Check that files were created
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(!entries.is_empty(), "No files downloaded");

    // Should have at least one media file and one info.json
    let has_media = entries.iter().any(|e| {
        let path = e.path();
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| ["jpg", "png", "gif", "webp"].contains(&s))
            .unwrap_or(false)
    });
    let has_info_json = entries
        .iter()
        .any(|e| e.path().to_str().unwrap().ends_with(".info.json"));

    println!("Downloaded files:");
    for entry in &entries {
        println!("  - {:?}", entry.path().file_name());
    }

    assert!(has_media, "No media file found");
    assert!(has_info_json, "No info.json found");
}

#[test]
#[ignore]
fn test_save_reddit_video_post() {
    // Reddit post with v.redd.it video
    let url = Url::parse("https://www.reddit.com/r/funny/comments/wge0pc/weatherman_finds_out_he_has_touch_screen_this/").unwrap();
    let analysis = save::analyze(&url).expect("analyze");
    assert_eq!(analysis.targets.len(), 1);

    println!("Reddit video post analysis:");
    println!("  Title: {:?}", analysis.title);
    println!("  Targets: {} ({:?})", analysis.targets.len(), analysis.targets[0]);

    let temp_dir = tempfile::tempdir().unwrap();
    save::download(&analysis.targets[0], temp_dir.path()).expect("download");

    // Check that files were created
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(!entries.is_empty(), "No files downloaded");

    // Should have at least a video file and an info.json
    let has_video = entries.iter().any(|e| {
        let path = e.path();
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| ["mp4", "webm", "mkv"].contains(&s))
            .unwrap_or(false)
    });
    let has_info_json = entries
        .iter()
        .any(|e| e.path().to_str().unwrap().ends_with(".info.json"));

    println!("Downloaded files:");
    for entry in &entries {
        println!("  - {:?}", entry.path().file_name());
    }

    assert!(has_video, "No video file found");
    assert!(has_info_json, "No info.json found");
}

#[test]
#[ignore]
fn test_save_reddit_gallery() {
    // Reddit gallery post (multiple images)
    let url = Url::parse("https://www.reddit.com/r/OldSchoolCool/comments/1hrtpo8/in_1974_masahisa_fukase_photographed_his_wife/").unwrap();
    let analysis = save::analyze(&url).expect("analyze");
    assert_eq!(analysis.targets.len(), 1);

    println!("Reddit gallery analysis:");
    println!("  Title: {:?}", analysis.title);
    println!("  Targets: {} ({:?})", analysis.targets.len(), analysis.targets[0]);

    let temp_dir = tempfile::tempdir().unwrap();
    save::download(&analysis.targets[0], temp_dir.path()).expect("download");

    // Check that files were created
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(!entries.is_empty(), "No files downloaded");

    // Count media files
    let media_files: Vec<_> = entries
        .iter()
        .filter(|e| {
            let path = e.path();
            path.extension()
                .and_then(|s| s.to_str())
                .map(|s| ["jpg", "png", "gif", "webp"].contains(&s))
                .unwrap_or(false)
        })
        .collect();

    let info_json_files: Vec<_> = entries
        .iter()
        .filter(|e| {
            let path = e.path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            filename.ends_with(".info.json") || filename == "info.json"
        })
        .collect();

    println!("Downloaded files:");
    for entry in &entries {
        println!("  - {:?}", entry.path().file_name());
    }

    // Should have multiple images
    assert!(
        media_files.len() > 1,
        "Should have multiple media files for a gallery (found {})",
        media_files.len()
    );
    assert_eq!(
        info_json_files.len(),
        1,
        "Should have exactly one info.json file for gallery"
    );
}

#[test]
#[ignore]
fn test_save_reddit_external_link() {
    // Reddit post linking to external image (e.g., imgur)
    let url = Url::parse("https://www.reddit.com/r/aww/comments/fnkcys/while_showing_my_3_year_old_my_game_boy_i/").unwrap();
    let analysis = save::analyze(&url).expect("analyze");
    assert_eq!(analysis.targets.len(), 1);

    println!("Reddit external link analysis:");
    println!("  Title: {:?}", analysis.title);
    println!("  Targets: {} ({:?})", analysis.targets.len(), analysis.targets[0]);

    let temp_dir = tempfile::tempdir().unwrap();
    save::download(&analysis.targets[0], temp_dir.path()).expect("download");

    // Check that files were created
    let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(!entries.is_empty(), "No files downloaded");

    // Should have at least one media file and one info.json
    let has_media = entries.iter().any(|e| {
        let path = e.path();
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| ["jpg", "png", "gif", "webp"].contains(&s))
            .unwrap_or(false)
    });
    let has_info_json = entries
        .iter()
        .any(|e| e.path().to_str().unwrap().ends_with(".info.json"));

    println!("Downloaded files:");
    for entry in &entries {
        println!("  - {:?}", entry.path().file_name());
    }

    assert!(has_media, "No media file found");
    assert!(has_info_json, "No info.json found");
}
