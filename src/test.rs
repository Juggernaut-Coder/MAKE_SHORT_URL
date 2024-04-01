use super::*;
use std::time::Duration;
use env_logger::Builder;
use futures::StreamExt;
use bb8_redis::redis::{Client, cmd};
use actix_web::{test, rt::time::sleep};
use log::info;
use redis_conn::redis_url;
use std::sync::Mutex;
use lazy_static::lazy_static;

// run test one by one in order in sync
lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

#[actix_web::test]
async fn test_redis() {
    let _guard = TEST_MUTEX.lock().unwrap();
    //tests redis connection, set command, pubsub and expiry behavior 
    let _ = Builder::new().filter_level(LevelFilter::Debug).is_test(true).try_init();
    let client = Client::open(redis_url()).expect("Failed to create Redis client");
    let mut conn = client.get_multiplexed_async_connection().await.expect("Failed to connect to Redis"); 
    let config: Vec<(String, String)> = cmd("CONFIG").arg("GET").arg("notify-keyspace-events").query_async(&mut conn).await.unwrap();
    if let Some((key, val)) = config.get(0) {
        info!("{}: {}", key, val);
        if !val.contains('x') || !val.contains('E') {
            cmd("CONFIG").arg("SET").arg("notify-keyspace-events").arg("xE").query_async::<_,()>(&mut conn).await.unwrap();
        }
    }
    cmd("SET").arg("testkey0").arg("testvalue").arg("EX").arg("30").query_async::<_,()>(&mut conn).await
    .expect("Failed to set key"); 
    cmd("SET").arg("testkey1").arg("notvalue").arg("EX").arg("30").query_async::<_,()>(&mut conn).await
    .expect("Failed to set key");
    let mut pubsub_conn = client.get_async_pubsub().await.unwrap();   
    pubsub_conn.psubscribe("__key*@*__:*").await
    .expect("FAILED TO LISTEN TO PSUBSCRIBE CHANNEL");
    let mut stream = pubsub_conn.into_on_message();
    sleep(Duration::from_secs(5)).await;
    let mut ttl: i64 = cmd("TTL").arg("testkey0").query_async(&mut conn).await
    .expect("Failed to get TTL");
    info!("TTL 1st val after 5 seconds: {}", ttl);
    ttl = cmd("TTL").arg("testkey1").query_async(&mut conn).await.expect("Failed to get TTL");
    info!("TTL 2nd val after 5 seconds: {}", ttl);
    sleep(Duration::from_secs(20)).await;
    let mut cnt = 0; 
    while let Some(message) = stream.next().await {
        cnt += 1;
        let payload: String = message.get_payload().unwrap();
        info!("Received message: {}", payload);
        if cnt == 2 {break;} // test for 2 msg
    }
    let exists: bool = cmd("EXISTS").arg("testkey0").query_async(&mut conn).await
    .expect("Failed to check if key exists");
    assert_eq!(exists, false);
    info!("Key1 exists after expiry: {}", exists);
    let exists: bool = cmd("EXISTS").arg("testkey1").query_async(&mut conn).await
    .expect("Failed to check if key exists");
    assert_eq!(exists, false);
    info!("Key2 exists after expiry: {}", exists);
}

#[actix_web::test]
async fn test_process_url() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let _ = Builder::new().filter_level(LevelFilter::Debug).is_test(true).try_init();
    let pool = get_redis_pool().await;
    let app = App::new()
        .app_data(web::Data::new(pool.clone()))
        .route("/shorten", web::post().to(process_url));

    let mut app = test::init_service(app).await;
    let test_url = "http://www.google.com";
    let req = test::TestRequest::post()
        .uri("/shorten")
        .set_form(&UrlForm { url: test_url.to_string() })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    info!("Testing URL: {}", test_url);
    info!("Status: {}", status);
    info!("Body: {}", String::from_utf8_lossy(&body));
    assert!(status.is_success(), "Expected success status, got {}. Body: {}", status, String::from_utf8_lossy(&body));
}

#[actix_web::test]
async fn test_url_redirection() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let _ = Builder::new().filter_level(LevelFilter::Debug).is_test(true).try_init();
    let encoded_id = "0";
    let pool = get_redis_pool().await;
    let mut conn = pool.get().await.unwrap();
    let test_url = "http://www.google.com";
    cmd("SET").arg(encoded_id).arg(test_url).arg("EX").arg(15).query_async::<_,()>(&mut *conn).await.unwrap();
    let app = test::init_service(
        App::new().route("/{short_url:.*}", web::get().to(short_url_redirect))
        .app_data(web::Data::new(pool.clone()))
    ).await;
    let req = test::TestRequest::with_uri(&format!("/{short_url}", short_url = encoded_id)).to_request();
    let resp = test::call_service(&app, req).await;
    use actix_web::http::StatusCode;
    assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
    sleep(Duration::from_secs(15)).await;
}

#[actix_web::test]
async fn test_redis_pool() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let pool = redis_conn::get_redis_pool().await;
    let redis_conn = pool.get().await;
    assert!(redis_conn.is_ok());
    let mut redis_conn = redis_conn.unwrap();
    let reply: String = cmd("PING").query_async(&mut *redis_conn).await.unwrap();
    assert_eq!("PONG", reply);
}

