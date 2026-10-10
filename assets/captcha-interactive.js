(() => {
  const config = window.__sunoCliCaptchaConfig;
  const apiName = config.provider === 2 ? 'turnstile' : 'hcaptcha';
  const previous = window.__sunoCliCaptchaWidget;
  if (previous) window[previous.api]?.remove(previous.id);
  document.getElementById('suno-cli-verification')?.remove();

  const state = { status: 'pending' };
  window.__sunoCliCaptchaState = state;
  const active = () => window.__sunoCliCaptchaState === state;
  const fail = reason => {
    if (!active()) return;
    state.status = 'error';
    state.error = reason;
    delete state.token;
  };
  const box = document.createElement('section');
  box.id = 'suno-cli-verification';
  box.style.cssText = 'position:fixed;top:24px;left:50%;transform:translateX(-50%);z-index:2147483647;background:white;color:#111;padding:24px;border:1px solid #888;border-radius:12px;box-shadow:0 6px 30px #0005;';
  const heading = document.createElement('h2');
  heading.textContent = 'Conclua a verificação para continuar';
  const message = document.createElement('p');
  message.textContent = 'Mantenha esta janela aberta. Ela será reutilizada nas próximas gerações deste lote.';
  const widget = document.createElement('div');
  box.append(heading, message, widget);
  document.body.appendChild(box);

  const options = {
    sitekey: config.sitekey,
    size: 'normal',
    callback: token => {
      if (active() && state.status === 'pending') {
        state.status = 'solved';
        state.token = token;
        message.textContent = 'Verificação concluída. A geração continuará automaticamente.';
      }
    },
    'error-callback': () => fail('challenge_error'),
    'expired-callback': () => fail('challenge_expired'),
    'timeout-callback': () => fail('challenge_timeout')
  };
  if (config.provider === 1) {
    Object.assign(options, {
      endpoint: 'https://hcaptcha-endpoint-prod.suno.com',
      assethost: 'https://hcaptcha-assets-prod.suno.com',
      imghost: 'https://hcaptcha-imgs-prod.suno.com',
      reportapi: 'https://hcaptcha-reportapi-prod.suno.com'
    });
  } else {
    options.appearance = 'always';
  }
  try {
    const id = window[apiName].render(widget, options);
    window.__sunoCliCaptchaWidget = { api: apiName, id };
  } catch (_) {
    fail('challenge_render_error');
  }
})();
