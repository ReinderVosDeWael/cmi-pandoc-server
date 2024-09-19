use std::io::prelude::Read;

use axum::{
    extract::DefaultBodyLimit,
    http::{self, StatusCode},
    response::IntoResponse,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use pandoc::{InputKind, OutputKind};

use axum::routing;
use tempfile::NamedTempFile;
use tracing::{debug, info};

pub(crate) fn init_router() -> axum::Router {
    axum::Router::new()
        .route("/pandoc", routing::post(post_pandoc))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
}

#[derive(TryFromMultipart)]
struct UploadDocumentRequest {
    #[form_data(limit = "unlimited")]
    file: FieldData<NamedTempFile>,
    to: String,
    from: String,
}

async fn post_pandoc(
    TypedMultipart(UploadDocumentRequest { mut file, to, from }): TypedMultipart<
        UploadDocumentRequest,
    >,
) -> Result<impl IntoResponse, http::StatusCode> {
    info!("Entering POST pandoc endpoint.");
    let file_contents =
        convert_file_to_bytes(&mut file.contents).map_err(|_| return StatusCode::BAD_REQUEST)?;

    match run_pandoc(to, from, file_contents) {
        Ok(value) => Ok(value),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn run_pandoc(to: String, from: String, file: bytes::Bytes) -> std::io::Result<Vec<u8>> {
    debug!("Running pandoc.");
    let input_file = tempfile::Builder::new()
        .suffix(&format!(".{}", from))
        .tempfile()?;

    std::fs::write(input_file.path(), file)?;

    let output_file = tempfile::Builder::new()
        .suffix(&format!(".{}", to))
        .tempfile()?;

    let mut pandoc = pandoc::Pandoc::new();
    pandoc.set_input(InputKind::Files(vec![input_file.path().to_path_buf()]));
    pandoc.set_output(OutputKind::File(output_file.path().to_path_buf()));
    pandoc
        .execute()
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

    let mut file = std::fs::File::open(output_file.path())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn convert_file_to_bytes(file: &mut NamedTempFile) -> std::io::Result<bytes::Bytes> {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(bytes::Bytes::from(buffer))
}
