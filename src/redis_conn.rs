#![allow(clippy::needless_return)]
#![allow(clippy::explicit_auto_deref)]

#[cfg(debug_assertions)]
pub const TTL: i64 = 60;
#[cfg(not(debug_assertions))]
pub const TTL: i64 = (24 * 60 * 60) + 30;
pub const URL_ID: &str = "url_id";
pub const ID_LIST: &str = "id_list";
pub const HASH_MAP: &str = "url_hash_map"; 

use bb8_redis::{bb8::Pool, redis::{Client, aio, cmd}, RedisConnectionManager};
use futures::StreamExt;
use actix_web::{web, HttpResponse, Responder};
use log::debug;

//Under normal circumstances api keys should never be exposed like this!
const END_POINT: &str = "redis-13989.c329.us-east4-1.gce.cloud.redislabs.com:13989";
const USR: &str = "Syed";
const PASS: &str = "Uncertainty320@@";

pub fn redis_url() -> String {
    return format!("redis://{}:{}@{}", USR, PASS, END_POINT);
}

pub async fn get_redis_pool() -> Pool<RedisConnectionManager> {
    let manager = RedisConnectionManager::new(redis_url()).expect("Failed to create Redis connection manager");
    return Pool::builder().build(manager).await.unwrap();
}

pub async fn setup_redis(redis_conn: &mut aio::MultiplexedConnection) {
    cmd("SET").arg(URL_ID).arg("1").arg("NX").query_async::<_,()>(redis_conn).await.unwrap();
    let exists_id_list: i32 = cmd("EXISTS").arg(ID_LIST).query_async(redis_conn).await.unwrap();
    if exists_id_list != 1 { cmd("LPUSH").arg(ID_LIST).arg("i0").query_async::<_,()>(redis_conn).await.unwrap(); }
    let config: Vec<(String, String)> = cmd("CONFIG").arg("GET").arg("notify-keyspace-events").query_async(redis_conn).await.unwrap();
    if let Some((key, val)) = config.first() {
        debug!("{}: {}", key, val);
        if !val.contains('E') || !val.contains('x') {
            cmd("CONFIG").arg("SET").arg("notify-keyspace-events").arg("Ex").query_async::<_,()>(redis_conn).await.unwrap();
        }
    }
}

pub async fn redis_flush_db(pool: web::Data<Pool<RedisConnectionManager>>) -> impl Responder {
    let mut conn = pool.get().await.unwrap();
    cmd("FLUSHDB").query_async::<_,()>(&mut *conn).await.unwrap();
    setup_redis(&mut *conn).await;
    return HttpResponse::Ok().body("Redis FLUSHDB executed successfully");
}

pub async fn redis_psub_expiry() {
    debug!("Running pubsub background task");
    let client = Client::open(redis_url()).unwrap();
    let mut multiplex_conn = client.get_multiplexed_async_connection().await.unwrap();
    let mut conn = client.get_async_pubsub().await.unwrap();
    conn.psubscribe("__keyevent@0__:expired").await.unwrap();
    let mut pubsub_stream = conn.into_on_message();
    while let Some(message) = pubsub_stream.next().await {
        let encoded_id: String = message.get_payload().unwrap();
        if !encoded_id.contains('/') {
            debug!("Expired ID: {}", encoded_id);
            cmd("RPUSH").arg(ID_LIST).arg(encoded_id.as_str()).query_async::<_, ()>(&mut multiplex_conn).await.unwrap();    
        }      
    }
}
