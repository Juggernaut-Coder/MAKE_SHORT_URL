#![allow(non_snake_case)]
#![allow(clippy::needless_return)]
#![allow(clippy::explicit_auto_deref)]

pub mod base62;
pub use base62::encode;
pub use redis_conn::{TTL, ID_LIST, URL_ID, redis_psub_expiry, get_redis_pool, setup_redis, redis_flush_db};
pub use env_logger::Builder;
pub use log::{LevelFilter, debug, warn};
pub use serde::{Deserialize, Serialize};
pub use reqwest::Url;
pub use actix_web::{web, rt, App, HttpResponse, HttpServer, middleware::Logger};
pub use actix_files::{NamedFile, Files};
pub use bb8_redis::{bb8::Pool, redis::cmd, RedisConnectionManager};

#[derive(Debug, Deserialize, Serialize)]
pub struct UrlForm {
    url: String
}

#[derive(Serialize)]
struct UrlResponse {
    msg: String
}

#[actix_web::main]
async fn main() {
    //Setup logging
    #[cfg(debug_assertions)]
    Builder::new().filter_level(LevelFilter::Debug).init();
    #[cfg(not(debug_assertions))]
    Builder::new().filter_level(LevelFilter::Warn).init();
    //Set up Redis
    let redis_pool = get_redis_pool().await;
    let mut redis_conn = redis_pool.get().await.unwrap();
    setup_redis(&mut *redis_conn).await;
    drop(redis_conn);
    //Setup server
    let server = HttpServer::new( move || {
        let mut app = App::new()
        .wrap(Logger::default())
        .app_data(web::Data::new(redis_pool.clone()))
        .route("/", web::get().to(redirect_to_home))
        .route("/home", web::get().to(homepage))
        .route("/shorten", web::post().to(process_url));
        #[cfg(debug_assertions)] {
            app = app.service(Files::new("/static", "static")
                .show_files_listing()
                .use_last_modified(false)
                .use_etag(false))
                .route("/flushdb", web::delete().to(redis_flush_db)); /* To use this from a terminal -> curl -X DELETE http://us.ex/flushdb */
        }
        #[cfg(not(debug_assertions))] {
            app = app.service(Files::new("/static", "static")
                .use_last_modified(true)
                .use_etag(true));
        }
        app = app.route("/{short_url:.*}", web::get().to(short_url_redirect));
        app
    }).bind("0.0.0.0:80").unwrap().shutdown_timeout(30).run(); 
    /* To run this demo it is expected that (unix/linux) /etc/hosts or (windows) C:\Windows\System32\drivers\etc\hosts contains -> 127.0.0.1 us.ex */
    rt::spawn(redis_psub_expiry());
    server.await.unwrap();
}

pub async fn homepage() -> actix_web::Result<NamedFile> {
    return Ok(NamedFile::open("static/home.html")?);
}

pub async fn redirect_to_home() -> HttpResponse {
    return HttpResponse::Found().append_header(("Location", "/home")).finish();
}

pub async fn process_url(pool: web::Data<Pool<RedisConnectionManager>>, form: web::Form<UrlForm>) -> HttpResponse {
    let input_url = form.url.trim();
    debug!("Received URL: {}", input_url);
    let mut response = "Invalid URL format! Try again".to_string();
    let rqst_url = Url::parse(input_url);
    if rqst_url.is_ok() {
        let ping_result = reqwest::get(rqst_url.unwrap()).await;
        if ping_result.is_ok() {
            debug!("URL is valid");
            let mut redis_conn = pool.get().await.unwrap();
            let mut set: bool = false;
            debug!("Checking if URL already in DB");
            let mut encoded_id = cmd("GET").arg(input_url).query_async::<_, Option<String>>(&mut *redis_conn).await.unwrap_or(None);
            if encoded_id.is_some() { //set = false;
                debug!("URL is in DB! Extending by TTL from current time");
                cmd("EXPIRE").arg(input_url).arg(TTL).query_async::<_,()>(&mut *redis_conn).await.unwrap();
            } else {
                debug!("Checking if List has any available ID");
                encoded_id = cmd("LPOP").arg(ID_LIST).query_async::<_, Option<String>>(&mut *redis_conn).await.unwrap_or(None);
                if encoded_id.is_none() {
                    debug!("List is empty! Generating new ID");
                    let id: u64 = cmd("GET").arg(URL_ID).query_async(&mut *redis_conn).await.unwrap();
                    // Naive overflow check, in real world scenerio maybe incoporate UUID or other mechanism
                    if id > u64::MAX - 5 {
                        warn!("Cannot Generate a new ID! ID capacity at its maximum limit");
                        return HttpResponse::Ok().json(UrlResponse { msg: "ID at max capacity. Please Wait Until Other ids free up".to_string() });
                    } else {
                        debug!("Generated id:{}", id);
                        cmd("INCR").arg(URL_ID).query_async::<_,()>(&mut *redis_conn).await.unwrap();
                        encoded_id = Some(encode(id).await);
                    }
                }
                set = true;
            }
            let encoded_id = encoded_id.unwrap();
            debug!("Encoded ID: {}", encoded_id);
            if set {
                debug!("Creating new entry for DB");
                cmd("SET").arg(encoded_id.as_str()).arg(input_url).arg("EX").arg(TTL).query_async::<_,()>(&mut *redis_conn).await.unwrap(); 
                cmd("SET").arg(input_url).arg(encoded_id.as_str()).arg("EX").arg(TTL).query_async::<_,()>(&mut *redis_conn).await.unwrap();                
            }
            response = format!("http://us.ex/{}", encoded_id);
        } else { response = "URL not reachable! Try again".to_string(); }
    }
    return HttpResponse::Ok().json(UrlResponse { msg: response });
}

pub async fn short_url_redirect(pool: web::Data<Pool<RedisConnectionManager>>, short_url: web::Path<String>) -> HttpResponse { 
    debug!("Redirecting");
    let short_url = short_url.into_inner();
    let mut redis_conn = pool.get().await.unwrap();
    let long_url: Option<String> = cmd("GET").arg(short_url).query_async::<_, Option<String>>(&mut *redis_conn).await.unwrap_or(None);
    match long_url {
        Some(url) => HttpResponse::TemporaryRedirect()
            .append_header(("Location", url))
            .finish(),
        None => HttpResponse::NotFound().body("Short URL not found"),
    }
}

#[cfg(test)]
mod test;




