use std::io::Write;

use actix_web::{App, HttpResponse, HttpServer, web, Error};
use actix_multipart::{Multipart, form::{MultipartForm, tempfile::TempFile, text::Text, MultipartFormConfig}};
use futures_util::{TryStreamExt};

#[derive(Debug, MultipartForm)]
pub struct FormularioArchivos {
    pub nombre:Text<String>,
    pub apellido: Text<String>,
    pub avatar: TempFile,
}

async fn formulario_multipart(MultipartForm(form): MultipartForm<FormularioArchivos>) -> Result<HttpResponse, Error>{
    println!("Nombre: {}", *form.nombre);
    println!("apellido: {}", *form.apellido);
    let file_name = form.avatar.file_name.unwrap();

    let file_path = format!("./archivos/{file_name}");

    form.avatar.file.persist(file_path).unwrap();

    Ok(HttpResponse::Ok().body(format!("Recibido: {file_name}")))
}

async fn upload(mut payload:Multipart) -> Result<HttpResponse, Error>{
    while let Some(mut field) = payload.try_next().await? {
        // 1) content_disposition() devuelve Option
        let Some(content_disposition) = field.content_disposition() else {
            continue;
        };

        // 2) get_filename() también devuelve Option
        let Some(file_name) = content_disposition.get_filename() else {
            continue;
        };

        let file_path = format!("./archivos/{file_name}");

        // 3) Crear el archivo en un bloque (I/O bloqueante)
        let mut archivo = web::block(move || std::fs::File::create(&file_path))
            .await
            .map_err(actix_web::error::ErrorInternalServerError)??;

        // 4) Escribir cada chunk
        while let Some(chunk) = field.try_next().await? {
            archivo = web::block(move || archivo.write_all(&chunk).map(|_| archivo))
                .await
                .map_err(actix_web::error::ErrorInternalServerError)??;
        }
    }

    Ok(HttpResponse::Ok().body("Archivo subido correctamente!"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let multipartform_config = MultipartFormConfig::default().total_limit(1024 * 1024 * 100).memory_limit(1024 * 1024 * 100);
        App::new()
        .app_data(multipartform_config)
        .route("archivosmultipart", web::post().to(formulario_multipart))
        .route("subirarchivo", web::post().to(upload))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
