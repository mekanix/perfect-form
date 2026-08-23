use topcoat::{
    Result,
    asset::{Asset, asset},
    view::{View, component, view},
};

pub(crate) const ROOT_CSS: Asset = asset!("assets/root.css");

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
                <script
                    src="https://cdn.jsdelivr.net/npm/htmx-ext-head-support@2.0.5"
                ></script>
                topcoat::dev::script()
            </head>
            <body hx-boost="true" hx-ext="head-support">
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
