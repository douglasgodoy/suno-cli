mod common;

use common::{agent_info, suno_in};

#[test]
fn cover_exposes_controls_in_machine_readable_discovery() {
    let manifest = agent_info();
    let options = manifest["commands"]["cover"]["options"].as_array().unwrap();
    for flag in [
        "--title",
        "--exclude",
        "--vocal",
        "--weirdness",
        "--style-influence",
        "--dry-run",
    ] {
        assert!(
            options.iter().any(|o| o["name"] == flag),
            "cover is missing {flag}"
        );
    }
}

#[test]
fn cover_accepts_all_skill_controls_before_authentication() {
    let home = tempfile::tempdir().unwrap();
    let out = suno_in(home.path())
        .args([
            "cover",
            "source-id",
            "--title",
            "Song (Dub Remix)",
            "--tags",
            "roots dub",
            "--exclude",
            "reggaeton dembow",
            "--vocal",
            "male",
            "--weirdness",
            "20",
            "--style-influence",
            "70",
            "--audio-influence",
            "70",
            "--model",
            "v6",
            "--no-browser",
        ])
        .output()
        .unwrap();
    // An isolated account has no credentials. Parsing must succeed first.
    assert_eq!(out.status.code(), Some(2));
    let error: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_ne!(error["error"]["code"], "invalid_input");
}

#[test]
fn cover_rejects_invalid_slider_values_before_authentication() {
    let home = tempfile::tempdir().unwrap();
    for flag in ["--weirdness", "--style-influence", "--audio-influence"] {
        for value in ["-1", "100.1", "NaN", "inf"] {
            let out = suno_in(home.path())
                .args(["cover", "source-id", &format!("{flag}={value}")])
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(3), "{flag}={value}");
            let error: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
            assert!(
                error["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("finite number")
            );
        }
    }
}
