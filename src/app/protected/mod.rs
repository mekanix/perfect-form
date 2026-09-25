use topcoat::{
    Result,
    context::{Cx, app_context},
    router::route,
    view::{ViewExt, view},
};

use crate::{auth, auth::permissions, components::skeleton, config::Config};

#[permissions]
#[route(GET "/protected")]
pub async fn protected(cx: &Cx) -> Result<auth::Page> {
    let cfg: &Config = app_context(cx);
    let db: &toasty::Db = app_context(cx);

    let Some(user) = auth::current_user(cx, db, cfg).await? else {
        return Ok(auth::Page::View(
            view! { skeleton(title: "Protected", <h1>"Not found"</h1>) }
                .single()
                .await?,
        ));
    };

    Ok(auth::Page::View(
        view! {
            skeleton(
                title: "Protected",
                <h1>"Protected page"</h1>
                <p>
                    "Logged in as: "
                    (&user.email)
                </p>
            )
        }
        .single()
        .await?,
    ))
}
