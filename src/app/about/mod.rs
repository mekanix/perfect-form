use topcoat::{
    Result,
    asset::{Asset, asset},
    router::page,
    view::view,
};

use crate::components::skeleton;

const ABOUT_CSS: Asset = asset!("./style.css");

#[page]
pub async fn about() -> Result {
    view! {
        skeleton(
            title: "About",
            extra_head: view! { <link rel="stylesheet" href=(ABOUT_CSS) blocking="render"> }?,
            <h1>"About"</h1>
            <a href="/">"Home"</a>
        )
    }
}
