mod common;
use common::suno_in;

#[test]
fn multi_round_preview_is_offline_and_uses_stable_distinct_request_ids() {
    let home = tempfile::tempdir().unwrap();
    let args = [
        "generate",
        "--title",
        "Batch",
        "--lyrics",
        "A complete verse",
        "--model",
        "v6",
        "--rounds",
        "2",
        "--request-id",
        "11111111-1111-4111-8111-111111111111",
        "--dry-run",
        "--no-browser",
    ];
    let preview = || {
        let out = suno_in(home.path()).args(args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()
    };
    let a = preview();
    let b = preview();
    assert_eq!(a["data"]["submitted"], false);
    let requests = a["data"]["requests"].as_array().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0]["transaction_uuid"],
        "11111111-1111-4111-8111-111111111111"
    );
    assert_ne!(
        requests[0]["transaction_uuid"],
        requests[1]["transaction_uuid"]
    );
    assert_eq!(
        requests[1]["transaction_uuid"],
        b["data"]["requests"][1]["transaction_uuid"]
    );
    for req in requests {
        assert_eq!(req["prompt"], "A complete verse");
        assert!(req["token"].is_null());
    }
}

#[test]
fn rounds_rejects_zero_and_excessive_counts_before_authentication() {
    let home = tempfile::tempdir().unwrap();
    for command in ["generate", "cover"] {
        for count in ["0", "11"] {
            let mut args = vec![command];
            if command == "cover" {
                args.push("source-id");
            }
            args.extend(["--rounds", count]);
            let out = suno_in(home.path()).args(args).output().unwrap();
            assert_eq!(out.status.code(), Some(3));
        }
    }
}

#[test]
fn one_token_cannot_be_reused_for_multiple_rounds() {
    let home = tempfile::tempdir().unwrap();
    for command in ["generate", "cover"] {
        let mut args = vec![command];
        if command == "cover" {
            args.push("source-id");
        }
        args.extend(["--rounds", "2", "--token", "single-use", "--no-browser"]);
        let out = suno_in(home.path()).args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(3));
        let error: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains("single-use")
        );
    }
}
