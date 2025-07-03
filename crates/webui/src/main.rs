#[macro_use]
extern crate rocket;

use mtk::Vault;
use webui::{entry, files};

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
            ],
        )
        .mount(prefix, routes![files::static_index_js, files::static_index_css])
        .mount(prefix, routes![files::raw_file_get, files::raw_file_head])
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
