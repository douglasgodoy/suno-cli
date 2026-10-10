use super::types::{Clip, ControlSliders, GenerateRequest};

/// Build the request from the actual source clip, preserving its lyrics and
/// instrumental status. Cover's v2-web mapping still needs live validation;
/// a correct request does not guarantee musical fidelity.
pub(crate) fn build_cover_request(
    source: &Clip,
    model_key: &str,
    title: Option<&str>,
    tags: Option<&str>,
    exclude: Option<&str>,
    control_sliders: Option<ControlSliders>,
) -> GenerateRequest {
    let mut req = GenerateRequest::new(model_key, "cover");
    // Adapted from the title-resolution fix in paperfoot/suno-cli PR #6.
    req.title = title
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .or_else(|| Some(source.title.trim()).filter(|t| !t.is_empty()))
        .unwrap_or("Cover")
        .to_string();
    req.tags = tags
        .or(source.metadata.tags.as_deref())
        .unwrap_or_default()
        .to_string();
    req.prompt = source.metadata.prompt.clone().unwrap_or_default();
    req.make_instrumental = source.metadata.make_instrumental;
    req.negative_tags = exclude.unwrap_or_default().to_string();
    req.cover_clip_id = Some(source.id.clone());
    req.metadata.control_sliders = control_sliders;
    req
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> Clip {
        serde_json::from_value(serde_json::json!({
            "id": "source-id", "title": "Original song", "status": "complete",
            "model_name": "chirp-hawk", "created_at": "2026-10-10T00:00:00Z",
            "metadata": {"tags": "Brazilian roots reggae", "prompt": "[Verse]\nOriginal lyrics",
                "make_instrumental": false}
        }))
        .unwrap()
    }

    #[test]
    fn cover_explicit_vocal_replaces_conflicting_source_direction() {
        use crate::cli::VocalGender;
        for (tags, vocal, expected) in [
            (
                "reggae, warm female vocals",
                VocalGender::Male,
                "reggae, male vocals",
            ),
            ("dub, male voice", VocalGender::Female, "dub, female vocals"),
        ] {
            assert_eq!(
                crate::build_cover_tags(Some(tags), Some(&vocal)).as_deref(),
                Some(expected)
            );
            assert_eq!(
                crate::build_cover_tags(Some(tags), None).as_deref(),
                Some(tags)
            );
        }
        assert_eq!(crate::build_cover_tags(Some(""), None).as_deref(), Some(""));
        assert_eq!(
            crate::build_cover_tags(Some("female vocals"), Some(&VocalGender::Male)).as_deref(),
            Some("male vocals")
        );
        assert_eq!(
            crate::build_cover_tags(None, Some(&VocalGender::Female)).as_deref(),
            Some("female vocals")
        );
    }

    #[test]
    fn cover_request_preserves_source_when_options_are_omitted() {
        let req = build_cover_request(&source(), "chirp-hawk", None, None, None, None);
        let wire = serde_json::to_value(&req).unwrap();
        assert_eq!(wire["title"], "Original song");
        assert_eq!(wire["prompt"], "[Verse]\nOriginal lyrics");
        assert_eq!(wire["tags"], "Brazilian roots reggae");
        assert_eq!(wire["cover_clip_id"], "source-id");
        assert_eq!(wire["metadata"]["create_mode"], "cover");
        assert_eq!(wire["negative_tags"], "");
        assert_eq!(wire["make_instrumental"], false);
        assert!(wire["metadata"].get("control_sliders").is_none());
    }

    #[test]
    fn cover_request_sends_overrides_without_rewriting_lyrics() {
        let sliders = crate::build_control_sliders(Some(20.0), Some(70.0), Some(70.0));
        let req = build_cover_request(
            &source(),
            "chirp-hawk",
            Some(" Song (Dub Remix) "),
            Some("roots dub, male vocals"),
            Some("reggaeton dembow"),
            sliders,
        );
        let wire = serde_json::to_value(req).unwrap();
        assert_eq!(wire["title"], "Song (Dub Remix)");
        assert_eq!(wire["tags"], "roots dub, male vocals");
        assert_eq!(wire["negative_tags"], "reggaeton dembow");
        assert_eq!(wire["prompt"], "[Verse]\nOriginal lyrics");
        assert_eq!(wire["cover_clip_id"], "source-id");
        assert_eq!(wire["mv"], "chirp-hawk");
        assert_eq!(
            wire["metadata"]["control_sliders"],
            serde_json::json!({
                "weirdness_constraint": 0.2, "style_weight": 0.7, "audio_weight": 0.7
            })
        );
    }

    #[test]
    fn cover_request_handles_blank_titles_and_instrumental_sources() {
        let mut source = source();
        source.title = "  ".into();
        source.metadata.make_instrumental = true;
        source.metadata.prompt = None;
        let req = build_cover_request(
            &source,
            "chirp-hawk",
            Some(" "),
            None,
            None,
            crate::build_control_sliders(Some(0.0), Some(100.0), None),
        );
        let wire = serde_json::to_value(req).unwrap();
        assert_eq!(wire["title"], "Cover");
        assert_eq!(wire["prompt"], "");
        assert_eq!(wire["make_instrumental"], true);
        assert_eq!(
            wire["metadata"]["control_sliders"],
            serde_json::json!({
                "weirdness_constraint": 0.0, "style_weight": 1.0
            })
        );
        source.title = "Original song".into();
        let req = build_cover_request(&source, "chirp-hawk", Some(" "), Some(""), None, None);
        assert_eq!(req.title, "Original song");
        assert_eq!(req.tags, "");
    }
}
