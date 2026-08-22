use topcoat::{
    Result,
    asset::{Asset, asset},
    view::{View, component, view},
};

pub(crate) const ROOT_CSS: Asset = asset!("assets/root.css");
pub(crate) const TOAST_JS: Asset = asset!("assets/toast.js");

#[component]
pub async fn skeleton(title: &str, #[default] extra_head: View, child: View) -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8">
                <title>(title)</title>
                <link rel="stylesheet" href="https://unpkg.com/chota@latest">
                <link rel="stylesheet" href=(ROOT_CSS)>
                (extra_head)
                <script
                    src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js"
                ></script>
                <script src=(TOAST_JS)></script>
                topcoat::dev::script()
            </head>
            <body hx-boost="true">
                <nav>
                    <a href="/">"Home"</a>
                    " | "
                    <a href="/about">"About"</a>
                    " | "
                    <a href="/protected">"Protected"</a>
                    " | "
                    <a href="/admin">"Admin"</a>
                    " | "
                    <a href="/login">"Log in"</a>
                </nav>
                <main id="main">(child)</main>
                <div id="toast"></div>
            </body>
        </html>
    }
}
