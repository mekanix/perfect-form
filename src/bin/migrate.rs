#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = perfect_form::config::Config::from_environment(
        perfect_form::config::Environment::Development,
        None,
    );

    let db = toasty::Db::builder()
        .models(toasty::models!(
            perfect_form::entity::user::Model,
            perfect_form::entity::role::Model,
            perfect_form::entity::users_roles::Model
        ))
        .connect(&cfg.database_url)
        .await?;

    let config = toasty_cli::Config::load_or_default(std::path::Path::new("."))?;
    toasty_cli::ToastyCli::with_config(db, config)
        .parse_and_run()
        .await?;
    Ok(())
}
