//! Sequential submissions reuse the process-owned CAPTCHA browser, never tokens.
use crate::{
    api::{
        SunoClient,
        types::{Clip, GenerateRequest},
    },
    errors::CliError,
    output::{self, OutputFormat},
};
use sha2::{Digest, Sha256};

pub(crate) struct Options {
    pub token: Option<String>,
    pub provider: Option<u8>,
    pub no_captcha: bool,
    pub headless: bool,
    pub no_browser: bool,
    pub quiet: bool,
    pub resume_wait: Option<(u64, u64)>,
}

pub(crate) fn validate(rounds: u8, token: Option<&str>) -> Result<(), CliError> {
    if rounds > 1 && token.is_some() {
        return Err(CliError::InvalidInput(
            "CAPTCHA tokens are single-use; --token requires --rounds 1".into(),
        ));
    }
    Ok(())
}

/// Round zero retains existing single-request recovery semantics. Further IDs
/// are deterministic, so a rerun cannot silently resubmit a completed round.
pub(crate) fn round_id(base: &str, round: u8) -> String {
    if round == 0 {
        return base.to_string();
    }
    let digest = Sha256::digest(format!("suno-batch:{base}:{round}"));
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80; // UUID v8, application-defined deterministic ID
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}

pub(crate) fn preview(
    req: &GenerateRequest,
    rounds: u8,
    fmt: OutputFormat,
) -> Result<(), CliError> {
    if rounds == 1 {
        return super::generation::preview_request(req, fmt);
    }
    let mut requests = Vec::new();
    for round in 0..rounds {
        let mut value = serde_json::to_value(req)?;
        value["transaction_uuid"] = round_id(&req.transaction_uuid, round).into();
        requests.push(value);
    }
    let data = serde_json::json!({"dry_run":true,"submitted":false,"model_verified_live":false,
        "rounds":rounds,"batch_request_id":req.transaction_uuid,"requests":requests,
        "next_action":"Remove --dry-run; preserve --request-id and arguments when resuming"});
    match fmt {
        OutputFormat::Json => output::json::success(data),
        OutputFormat::Table => {
            println!("{}", serde_json::to_string_pretty(&data)?);
            Ok(())
        }
    }
}

fn failure(error: CliError, base: &str, round: u8, clips: &[Clip]) -> CliError {
    let cause = error.details();
    let ids: Vec<_> = clips.iter().map(|c| &c.id).collect();
    CliError::Recovery {
        source: Box::new(CliError::Diagnostic {
            source: Box::new(error),
            details: serde_json::json!({
                "batch_request_id":base,"round":round+1,"request_id":round_id(base,round),
                "completed_clip_ids":ids,"cause":cause
            }),
        }),
        suggestion: format!(
            "Inspect `suno jobs` and the library before resubmitting an uncertain request. Resume with the same arguments and --request-id {base}; completed rounds reuse their saved IDs."
        ),
    }
}

pub(crate) async fn submit(
    c: &SunoClient,
    req: &mut GenerateRequest,
    rounds: u8,
    options: Options,
) -> Result<Vec<Clip>, CliError> {
    validate(rounds, options.token.as_deref())?;
    let base = req.transaction_uuid.clone();
    let mut clips = Vec::new();
    for round in 0..rounds {
        req.transaction_uuid = round_id(&base, round);
        req.token = None;
        req.token_provider = None;
        let result = async {
            if let Some(ids) = super::jobs::existing(req)? {
                let result = if let Some((timeout, interval)) = options.resume_wait {
                    c.poll_clips(&ids, timeout, interval).await
                } else {
                    c.get_existing_clips(&ids).await
                };
                return result.map_err(|e| super::generation::generation_recovery(e, &ids));
            }
            c.validate_generation(req).await?;
            if !options.quiet {
                eprintln!(
                    "Submission {}/{rounds} (request {})",
                    round + 1,
                    req.transaction_uuid
                );
            }
            (req.token, req.token_provider) = crate::resolve_captcha(
                c,
                options.token.clone(),
                options.provider,
                options.no_captcha,
                options.headless,
                options.no_browser,
                options.quiet,
            )
            .await?;
            c.generate(req).await
        }
        .await;
        match result {
            Ok(mut round_clips) => clips.append(&mut round_clips),
            Err(e) => return Err(failure(e, &base, round, &clips)),
        }
    }
    Ok(clips)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_ids_are_stable_distinct_and_valid() {
        let base = "11111111-1111-4111-8111-111111111111";
        assert_eq!(round_id(base, 0), base);
        let ids: std::collections::HashSet<_> = (0..10).map(|r| round_id(base, r)).collect();
        assert_eq!(ids.len(), 10);
        for id in ids {
            assert!(uuid::Uuid::parse_str(&id).is_ok());
        }
        assert_eq!(round_id(base, 4), round_id(base, 4));
    }
    #[test]
    fn partial_failure_retains_batch_and_original_error_details() {
        let e = failure(
            CliError::RateLimitedFor {
                seconds: 42,
                suggestion: "wait".into(),
            },
            "batch",
            1,
            &[],
        );
        assert_eq!(e.exit_code(), 4);
        let d = e.details().unwrap();
        assert_eq!(d["batch_request_id"], "batch");
        assert_eq!(d["round"], 2);
        assert_eq!(d["cause"]["retry_after_secs"], 42);
    }
}
