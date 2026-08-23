use topcoat::{
    Result,
    context::{Cx, app_context},
    router::route,
    view::view,
};

use crate::{auth, auth::permissions, components::skeleton, config::Config};

#[permissions(all("admin"))]
#[route(GET "/admin")]
pub async fn admin(cx: &Cx) -> Result<auth::Page> {
    let cfg: &Config = app_context(cx);
    let db: &sea_orm::DatabaseConnection = app_context(cx);

    let Some(user) = auth::current_user(cx, db, cfg).await? else {
        return Ok(auth::Page::View(
            view! { skeleton(title: "Admin", <h1>"Not found"</h1>) }?,
        ));
    };

    Ok(auth::Page::View(view! {
        skeleton(
            title: "Admin",
            <h1>"Admin page"</h1>
            <p>
                "Welcome, "
                (&user.email)
            </p>
        )
    }?))
}
