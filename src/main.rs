#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use std::io::{self, stdin};
use url::Url;
use reqwest::blocking::Client;
use redis::{Commands, RedisResult};
use std::env;

fn main() {
    let rqwst_client = Client::new();
    println!("Please enter a URL to shorten:");
    let url = read_and_check_url(&rqwst_client);
    let redis_client = get_redis_client();
    
   
    


}

fn shorten_url(url : String) {

}

fn get_redis_client() -> redis::Client {
    /* these should be encrypted or set as environment variable or other secure ways */
    /* but of the purpose anyone being able to compile it the API Keys are left unsecured */
    let endpoint = "redis-13989.c329.us-east4-1.gce.cloud.redislabs.com:13989";
    let usr = "default";
    let pass = "evjcDeVcHRXiZ8wciXO9E7RWfVuk8CIz";
    let redis_url = format!("redis://{}:{}@{}", usr, pass, endpoint);
    let redis_client = redis::Client::open(redis_url).unwrap();
    return redis_client;
}

fn read_and_check_url(rqwst_client : &Client) -> String {
    let mut input_url = String::new();
    loop {
        input_url.clear();
        stdin().read_line(&mut input_url).unwrap();
        if Url::parse(&input_url).is_ok() {
            let res = rqwst_client.get(&input_url).send();
            if let Ok(response) = res {
                if response.status().is_success() {
                    println!("URL is valid. Please wait while we shorten the URL!...");
                    break;
                }
                else {println!("URL is accessible but returned a non-success status. Try again!");}
            } else {println!("Failed to make a request. Check your internet connection or URL and try again!");}
        } else {println!("Invalid url format. Try again!");}
    }
    return input_url;
}