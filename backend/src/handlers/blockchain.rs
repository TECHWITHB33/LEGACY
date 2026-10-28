use actix_web::HttpResponse;

pub async fn get_block() -> HttpResponse {
    HttpResponse::Ok().finish()
}
