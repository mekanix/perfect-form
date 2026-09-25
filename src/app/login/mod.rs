use serde::Deserialize;
use toasty::stmt::{List, Query};
use topcoat::{
    Result,
    context::{Cx, app_context},
    cookie::{Cookie, Cookies, SameSite, cookies, time},
    htmx::header::HX_REDIRECT,
    router::response::{IntoResponse, Response},
    router::{Body, content::Form, page, route},
    view::{View, ViewExt, ViewHandle, component, view},
};

use crate::{auth, components::skeleton, config::Config};

#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

enum LoginOutcome {
    Success,
    Error(ViewHandle),
}

impl IntoResponse for LoginOutcome {
    fn into_response(self, cx: &Cx) -> Result<Response> {
        match self {
            LoginOutcome::Success => Ok(Response::builder()
                .header(HX_REDIRECT, "/")
                .body(Body::empty())?),
            LoginOutcome::Error(view) => view.into_response(cx),
        }
    }
}

#[page]
pub async fn login() -> Result<impl View> {
    Ok(view! { skeleton(title: "Login", login_form(error: None, email: None, password: None)) })
}

#[route(POST "/login")]
pub async fn login_submit(cx: &Cx, Form(form): Form<LoginForm>) -> Result<LoginOutcome> {
    let cfg: &Config = app_context(cx);
    let db: &toasty::Db = app_context(cx);

    let mut db = db.clone();
    let user = Query::<List<crate::entity::user::Model>>::all()
        .filter(crate::entity::user::Model::fields().email().eq(&form.email))
        .first()
        .exec(&mut db)
        .await?;

    let Some(user) = user else {
        return Ok(LoginOutcome::Error(
            login_form_view(
                cx,
                Some("Invalid email or password".to_owned()),
                Some(form.email.clone()),
                Some(form.password.clone()),
            )
            .await?
            .single()
            .await?,
        ));
    };

    if auth::verify_password(&form.password, &user.password_hash).is_err() {
        return Ok(LoginOutcome::Error(
            login_form_view(
                cx,
                Some("Invalid email or password".to_owned()),
                Some(form.email),
                Some(form.password),
            )
            .await?
            .single()
            .await?,
        ));
    }

    let tokens = auth::generate_tokens(user.id, &cfg.jwt_secret)?;
    set_auth_cookies(cx, &tokens, cfg.cookie_secure());

    Ok(LoginOutcome::Success)
}

fn set_auth_cookies(cx: &Cx, tokens: &auth::Tokens, secure: bool) {
    let access = Cookie::build(("access_token", tokens.access_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(time::Duration::minutes(15))
        .build();

    let refresh = Cookie::build(("refresh_token", tokens.refresh_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(time::Duration::days(7))
        .build();

    cookies(cx).add(access);
    cookies(cx).add(refresh);
}

#[component]
async fn login_form(
    #[default] error: Option<String>,
    #[default] email: Option<String>,
    #[default] password: Option<String>,
) -> Result<impl View> {
    login_form_view(__cx, error, email, password).await
}

async fn login_form_view(
    cx: &Cx,
    error: Option<String>,
    email: Option<String>,
    password: Option<String>,
) -> Result<impl View> {
    let __cx = cx;
    let email_value = email.unwrap_or_default();
    let password_value = password.unwrap_or_default();
    Ok(view! {
        if let Some(error) = error {
            <div id="toast" class="visible" hx-swap-oob="true">(error)</div>
        }
        <h1>"Log in"</h1>
        <form hx-post="/login" hx-target="#main" hx-swap="innerHTML">
            <label>
                "Email"
                <input
                    type="email"
                    name="email"
                    required="true"
                    value=(email_value)
                    autofocus="true"
                >
            </label>
            <label>
                "Password"
                <input
                    type="password"
                    name="password"
                    required="true"
                    value=(password_value)
                >
            </label>
            <button type="submit">"Log in"</button>
        </form>
    })
}
