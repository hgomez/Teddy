use actix_web::http::header;
use actix_web::{dev, fs, error, multipart, HttpRequest, Error, Query, FutureResponse, HttpResponse, HttpMessage};
use std::path::Path;
use futures::future;
use futures::{Future, Stream};
use std::fs as sys_fs;
use std::io::Write;

pub fn welcome(_req: &HttpRequest) -> &'static str {
    "Welcome to Teddy, see ya !"
}

pub fn ping(_req: &HttpRequest) -> &'static str {
    "pong"
}

#[derive(Deserialize, Debug)]
pub struct DownloadQuery {
    path: String
}

pub fn download(query: Query<DownloadQuery>) -> Result<fs::NamedFile, Error> {
    debug!("Download handler for file {:?}", query);
    let file = fs::NamedFile::open(&query.path)?;
    let file_name = Path::new(&query.path).file_name()
        .and_then(|os_str| os_str.to_str().map(|r_str| String::from(r_str)))
        .map(|file_name| file_name.replace("\"", ""))
        .unwrap_or_else(|| String::from("Undefined"));
    let file = file.set_content_disposition(header::ContentDisposition {
        disposition: header::DispositionType::Attachment,
        parameters: vec![header::DispositionParam::Name(file_name)],
    });
    Ok(file)
}

pub fn upload(req: HttpRequest<()>) -> FutureResponse<HttpResponse> {
    Box::new(
        req.multipart()
            .map_err(error::ErrorInternalServerError)
            .map(handle_multipart_item)
            .flatten()
            .collect()
            .map(|file_name| HttpResponse::Ok().json(file_name))
    )
}

fn save_file(
    field: multipart::Field<dev::Payload>,
) -> Box<Future<Item=String, Error=Error>> {
    Box::new(
        future::result(
            field.content_disposition()
                .and_then(|cd|
                    cd.get_name()
                        .map(|v| String::from(v)))
                .ok_or(error::ErrorBadRequest("Missing name in multipart data")))
            .and_then(|file_name|
                sys_fs::File::create(file_name.clone())
                    .map_err(|e| error::ErrorInternalServerError(e))
                    .map(|file| (file_name, file)))
            .and_then(|(file_name, mut file)| {
                debug!("Saving {} file from upload", file_name);
                field
                    .fold(file_name, move |acc, bytes| {
                        let rt = file
                            .write_all(bytes.as_ref())
                            .map_err(|e| {
                                error!("Error saving file : {:?}", e);
                                error::MultipartError::Payload(error::PayloadError::Io(e))
                            });
                        future::result(rt.map(|_| acc))
                    })
                    .map_err(|e| {
                        error!("save_file failed, {:?}", e);
                        error::ErrorInternalServerError(e)
                    })
            })
    )
}

fn handle_multipart_item(
    item: multipart::MultipartItem<dev::Payload>,
) -> Box<Stream<Item=String, Error=Error>> {
    match item {
        multipart::MultipartItem::Field(field) => {
            Box::new(save_file(field).into_stream())
        }
        multipart::MultipartItem::Nested(mp) => Box::new(
            mp.map_err(error::ErrorInternalServerError)
                .map(handle_multipart_item)
                .flatten(),
        ),
    }
}