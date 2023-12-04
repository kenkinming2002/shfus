use rocket::fs::FileServer;
use rocket::fs::NamedFile;
use rocket::fs::TempFile;
use rocket::fs::relative;
use rocket::fs::Options;

use rocket::http::Status;

use rocket::post;

use rocket::form::FromForm;
use rocket::form::Form;

use std::path::Path;

#[derive(FromForm)]
struct Upload<'r> {
    files: Vec<TempFile<'r>>,
}

#[post("/upload", data = "<upload>")]
async fn upload(mut upload: Form<Upload<'_>>) -> Result<NamedFile, Status> {
    for file in &mut upload.files {
        let name = file.name().ok_or(Status::BadRequest)?.to_owned();
        let path = Path::new(relative!("uploads")).join(&name);
        file.move_copy_to(&path).await.ok().ok_or(Status::BadRequest)?;
    }
    NamedFile::open(relative!("statics/upload.html")).await.ok().ok_or(Status::InternalServerError)
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::new(relative!("/statics"), Options::Index))
        .mount("/", rocket::routes![upload])
}
