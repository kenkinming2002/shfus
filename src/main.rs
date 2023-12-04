use rocket::fs::FileServer;
use rocket::fs::TempFile;
use rocket::fs::relative;
use rocket::fs::Options;

use rocket::response;
use rocket::response::Response;
use rocket::response::Responder;

use rocket::http::Status;
use rocket::http::Header;
use rocket::http::ContentType;

use rocket::post;

use rocket::form::FromForm;
use rocket::form::Form;

use std::io::Cursor;
use std::path::Path;

#[derive(FromForm)]
struct Upload<'r> {
    files: Vec<TempFile<'r>>,
}

enum UploadResponder {
    BadUpload(&'static str),
    Success,
}

impl<'r> Responder<'r, 'static> for UploadResponder {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'static> {
        match self {
            Self::BadUpload(message) => Response::build()
                .status(Status::BadRequest)
                .header(ContentType::Plain)
                .sized_body(message.len(), Cursor::new(message))
                .ok(),
            Self::Success => Response::build()
                .status(Status::SeeOther)
                .header(Header::new("Location", "index.html"))
                .ok(),
        }
    }
}

#[post("/upload", data = "<upload>")]
async fn upload(mut upload: Form<Upload<'_>>) -> UploadResponder {
    for file in &mut upload.files {
        // 1: Sanitize the filename and preserve the file extension
        let name = match file.raw_name().ok_or(Status::BadRequest) {
            Ok(name) => name,
            Err(_) => return UploadResponder::BadUpload("Missing filename"),
        };

        let name = name.dangerous_unsafe_unsanitized_raw();
        let name = name.as_str();
        let name = name.chars().filter(|c| !std::path::is_separator(*c)).collect::<String>();
        if name.is_empty() {
            return UploadResponder::BadUpload("Empty filename");
        }

        // 2: Compute the upload path
        let path = Path::new(relative!("uploads"));
        let path = path.join(name);

        // 3: Persist the file
        if let Err(_) = file.move_copy_to(&path).await {
            return UploadResponder::BadUpload("Failed to persist some file");
        }

        // 4: Done
        eprintln!("File uploaded to {path}", path = path.display());
    }
    UploadResponder::Success
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::new(relative!("/statics"), Options::Index))
        .mount("/", rocket::routes![upload])
}
