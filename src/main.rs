#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

mod base62;
use base62::{encode, decode};
use std::io::{self, stdin};
use serde::{Deserialize, Serialize};
use reqwest::Url;
use bb8_redis::{bb8::{Pool, PooledConnection}, redis::{aio, cmd, AsyncCommands}, RedisConnectionManager};
use actix_web::{dev::Response, web, App, HttpResponse, HttpServer, Responder};
use actix_files::{NamedFile, Files};
const URL_ID_VAR_NAME: &str = "url_id_cnt";
const TTL : i64 = (24 * 60 * 60) + 30; // 1 day + 30 second grace time
const URL_ID: &str = "url_id";
const ID_LIST: &str = "id_list";

#[derive(Deserialize)]
struct UrlForm {
    url: String
}

#[derive(Serialize)]
struct UrlResponse {
    msg: String
}

#[actix_web::main]
async fn main() {
    //Set up Redis
    let redis_pool = get_redis_pool().await;
    let mut redis_conn = redis_pool.get().await.unwrap();
    setup_redis(&mut *redis_conn);
    drop(redis_conn);
    //Setup Server
    let server = HttpServer::new( move || {
        let mut app = App::new()
        .app_data(web::Data::new(redis_pool.clone()))
        .route("/", web::get().to(redirect_to_home))
        .route("/home", web::get().to(homepage))
        .route("/shorten", web::post().to(process_url));
        #[cfg(debug_assertions)] {
            app = app.service(Files::new("/static", "static")
                .show_files_listing()
                .use_last_modified(false)
                .use_etag(false))
                .route("/flushdb", web::delete().to(redis_flush_db)); /* To use this from a terminal curl -X DELETE http://us.ex/flushdb */
        }
        #[cfg(not(debug_assertions))] {
            app = app.service(Files::new("/static", "static")
                .use_last_modified(true)
                .use_etag(true));
        }
        app
    }).bind("0.0.0.0:80").unwrap(); 
    /* To run this demo it is expected that (unix/linux) /etc/hosts or (windows) C:\Windows\System32\drivers\etc\hosts contains = 127.0.0.1 us.ex */
    server.run().await.unwrap();
}

async fn homepage() -> actix_web::Result<NamedFile> {
    return Ok(NamedFile::open("static/home.html")?);
}

async fn redirect_to_home() -> HttpResponse {
    return HttpResponse::Found().append_header(("Location", "/home")).finish();
}

async fn process_url(pool: web::Data<Pool<RedisConnectionManager>>, form: web::Form<UrlForm>) -> HttpResponse {
    let input_url = form.url.trim();
    println!("Received URL: {}", input_url);
    let mut response = String::new();
    let rqst_url = Url::parse(&input_url);
    if rqst_url.is_ok() {
        let ping_result = reqwest::get(rqst_url.unwrap()).await;
        if let Ok(result) = ping_result {
            println!("URL is valid");
            let mut redis_conn = pool.get().await.unwrap();
            let mut id: Option<u64> = cmd("GET").arg(input_url).query_async(&mut *redis_conn).await.unwrap_or(None);
            if id.is_some() {
                cmd("EXPIRE").arg(input_url).arg(TTL).query_async::<_,()>(&mut *redis_conn).await.unwrap();
            } else {
                id = cmd("LPOP").arg(ID_LIST).query_async(&mut *redis_conn).await.unwrap_or(None);
                if id.is_none() {
                    id = Some(cmd("GET").arg(URL_ID).query_async(&mut *redis_conn).await.unwrap());
                    cmd("INCR").arg(URL_ID).query_async::<_,()>(&mut *redis_conn).await.unwrap();
                }
                if let Some(i) = id {
                    cmd("SET").arg(input_url).arg(i.to_string().as_str()).arg("EX").arg(TTL).query_async::<_,()>(&mut *redis_conn).await.unwrap(); 
                }
            }
            let id = id.unwrap();
            let encoded_id = encode(id).await;
            response = format!("http://us.ex/{}", encoded_id);
        } else { response = "URL not reachable! Try again".to_string(); }
    } else { response = "Invalid URL format! Try again".to_string(); }
    let serialied_response = UrlResponse { msg: response };
    return HttpResponse::Ok().json(serialied_response);
}

//redis
async fn setup_redis(redis_conn: &mut aio::MultiplexedConnection) {
    let reply: String = cmd("PING").query_async(redis_conn).await.unwrap();
    assert_eq!("PONG", reply);
    cmd("SETNX").arg(URL_ID).arg(0u64).query_async::<_,()>(redis_conn).await.unwrap();
    let exists_id_list: i32 = cmd("EXISTS").arg(ID_LIST).query_async(redis_conn).await.unwrap();
    if exists_id_list != 1 { cmd("LPUSH").arg(ID_LIST).arg("").query_async::<_,()>(redis_conn).await.unwrap(); }
}

async fn get_redis_pool() -> Pool<RedisConnectionManager> {
    /* these should be encrypted or set as environment variable or other secure ways should employed or code should be closed source */
    /* but to allow everyone to compile it, the API Keys are left unsecured */
    let endpoint = "redis-13989.c329.us-east4-1.gce.cloud.redislabs.com:13989";
    let usr = "Syed";
    let pass = "Uncertainty320@@";
    let redis_url = format!("redis://{}:{}@{}", usr, pass, endpoint);
    let manager = RedisConnectionManager::new(redis_url).expect("Failed to create Redis connection manager");
    return Pool::builder().build(manager).await.unwrap();
}

async fn redis_flush_db(pool: web::Data<Pool<RedisConnectionManager>>) -> impl Responder {
    // for debugging and clearing DB in realease it will not be implemented
    let mut conn = pool.get().await.unwrap();
    cmd("FLUSHDB").query_async::<_,()>(&mut *conn).await.unwrap();
    setup_redis(&mut *conn);
    return HttpResponse::Ok().body("Redis FLUSHDB executed successfully");
}
