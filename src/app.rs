use crate::{components::skeleton, config::Config};
use sea_orm::DatabaseConnection;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Router, RouterBuilderDiscoverExt, page},
    view::{component, view},
};

#[cfg(test)]
use topcoat::asset::{AssetConfig, Manifest, ManifestEntry};

mod about;
mod admin;
mod login;
mod protected;

pub fn router(db: DatabaseConnection, cfg: Config) -> Router {
    topcoat::router::module_router!()
        .discover()
        .assets(AssetBundle::load().unwrap())
        .cookies()
        .app_context(db)
        .app_context(cfg)
        .build()
}

#[cfg(test)]
fn test_asset_config() -> AssetConfig {
    AssetConfig::hosted_at(
        "/assets",
        Manifest {
            version: 1,
            assets: vec![ManifestEntry {
                id: crate::components::ROOT_CSS.id(),
                file: "root.css".to_owned(),
                hash: "0".to_owned(),
                content_type: "text/css".to_owned(),
            }],
        },
    )
}

/// Router used in unit tests, without the asset bundle so tests don't depend on
/// the build-time asset bundling step.
#[cfg(test)]
pub fn test_router(db: DatabaseConnection, cfg: Config) -> Router {
    topcoat::router::module_router!()
        .discover()
        .assets(test_asset_config())
        .cookies()
        .app_context(db)
        .app_context(cfg)
        .build()
}

#[page]
pub async fn home() -> Result {
    view! { skeleton(title: "Hello world", hello(name: "World")) }
}

#[component]
async fn hello(name: &str) -> Result {
    view! {
        <h1>
            "Hello, "
            (name)
            "!"
        </h1>
        <a href="/about">"About"</a>
    }
}
