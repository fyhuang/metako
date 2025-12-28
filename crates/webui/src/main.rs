#[macro_use]
extern crate rocket;

use mtk::Vault;
use webui::{entry, files, preview, history, query, edit, save};

fn mount_all_routes(
    builder: rocket::Rocket<rocket::Build>,
    prefix: &str,
) -> rocket::Rocket<rocket::Build> {
    builder
        .mount(
            prefix,
            routes![
                entry::view_entry,
                entry::view_entry_by_id,
                entry::index,
            ],
        )
        .mount(prefix, routes![preview::preview_get])
        .mount(
            prefix,
            routes![files::static_index_js, files::static_index_js_map, files::static_index_css],
        )
        .mount(prefix, routes![files::raw_file_get, files::raw_file_head])
        .mount(prefix, routes![files::generated_file_get])
        .mount(prefix, routes![query::surprise, query::search])
        .mount(
            prefix,
            routes![history::api_video_history, history::api_clear_history],
        )
        .mount(prefix, routes![edit::edit_entry])
        .mount(prefix, routes![save::download_url])
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let stash = Vault::from_cwd();

    let rocket_conf = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 7784));

    let builder = rocket::custom(rocket_conf).manage(stash);
    let b2 = mount_all_routes(builder, "/");

    let _rocket = b2.launch().await?;

    Ok(())
}
