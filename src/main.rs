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
        // 1: Sanitize the filename and preserve the file extension
        let name = file.raw_name().ok_or(Status::BadRequest)?;
        let name = name.dangerous_unsafe_unsanitized_raw();
        let name = name.as_str();
        let name = name.chars().filter(|c| !std::path::is_separator(*c)).collect::<String>();
        if name.is_empty() {
            return Err(Status::BadRequest);
        }

        // 2: Compute the upload path
        let path = Path::new(relative!("uploads"));
        let path = path.join(name);

        // 3: Persist the file
        file.move_copy_to(&path).await.ok().ok_or(Status::BadRequest)?;

        // 4: Done
        eprintln!("File uploaded to {path}", path = path.display());
    }
    NamedFile::open(relative!("statics/upload.html")).await.ok().ok_or(Status::InternalServerError)
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::new(relative!("/statics"), Options::Index))
        .mount("/", rocket::routes![upload])
}
