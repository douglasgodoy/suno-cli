use super::*;

fn response(state: &serde_json::Value) -> Result<Option<String>, CliError> {
    match state["status"].as_str() {
        Some("solved") => state["token"]
            .as_str()
            .filter(|t| !t.is_empty())
            .map(|t| Some(t.to_string()))
            .ok_or_else(|| CliError::Config("manual CAPTCHA returned an empty response".into())),
        Some("error") => Err(CliError::Config(
            "manual CAPTCHA failed or expired; no generation submitted".into(),
        )),
        _ => Ok(None),
    }
}

pub(super) async fn wait(
    ws: &mut CdpStream,
    next: &mut impl FnMut() -> u64,
    provider: u8,
) -> Result<String, CliError> {
    let config = serde_json::json!({"provider":provider,"sitekey":if provider==2 {
        "0x4AAAAAADI7xDNyj-3LcIbi"
    } else { SUNO_HCAPTCHA_SITEKEY }});
    let script = format!(
        "window.__sunoCliCaptchaConfig = {config};\n{}",
        include_str!("../../assets/captcha-interactive.js")
    );
    cdp_call(ws, next(), "Page.bringToFront", serde_json::json!({})).await?;
    let setup = cdp_call(
        ws,
        next(),
        "Runtime.evaluate",
        serde_json::json!({"expression":script,"returnByValue":true}),
    )
    .await?;
    if setup.get("exceptionDetails").is_some() {
        return Err(CliError::Config(
            "Could not display manual CAPTCHA; no generation submitted".into(),
        ));
    }
    solver_log!(
        "CAPTCHA: complete the verification in the open Chrome window (up to 3 minutes). This window is reused for the batch."
    );
    let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
    while tokio::time::Instant::now() < deadline {
        sleep(Duration::from_secs(1)).await;
        let result = cdp_call(ws,next(),"Runtime.evaluate",serde_json::json!({
            "expression":"window.__sunoCliCaptchaState || {status:'pending'}","returnByValue":true
        })).await?;
        if let Some(token) = response(&result["result"]["value"])? {
            return Ok(token);
        }
    }
    Err(CliError::Config(
        "manual CAPTCHA timed out after 3 minutes; no generation submitted".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_a_nonempty_solved_response_can_continue() {
        assert!(
            response(&serde_json::json!({"status":"pending"}))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            response(&serde_json::json!({"status":"solved","token":"fresh"})).unwrap(),
            Some("fresh".into())
        );
        assert!(response(&serde_json::json!({"status":"solved","token":""})).is_err());
        let e = response(&serde_json::json!({"status":"error","token":"secret","error":"secret"}))
            .unwrap_err();
        assert!(!e.to_string().contains("secret"));
    }
}
