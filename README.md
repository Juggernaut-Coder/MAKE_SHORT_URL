# MAKE_SHORT_URL
## Description:
MAKE_SHORT_URL is a web application that enables users to create short URLs, similar to services like TinyURL. It employs actix_web for its server backend, utilizing redis as its database, and is designed to run within a Docker container environment.
##Source Files:
1) base62.rs: Contains code to convert a numerical value into a short string using base62 encoding.
2) redis_conn.rs: This source files defines contants and functions for interacting with a redis database. The constants includes name of the data/datastructure. The functions includes functions for setting up redis connection, flushing the database and handling expiration events using Redis' pub/sub feature. It uses actix_web for web server functionalities, bb8_redis for Redis connection pooling, and employs async/await for asynchronous operation.
3) server.rs: This file is the main entry point for the web app. It integrates with a Redis database for URL management and uses Actix Web for the web server. It also includes debug and production configurations for logging and file serving.
4) test.rs: contains test cases for the project.
##Static Files:
1) handle_button.js: This JavaScript file adds an event listener to the form identified by 'urlForm' on the webpage and handles shorten button click. When the form is submitted, it sends a POST request to the server at the '/shorten' endpoint with the URL to be shortened. The server response is then displayed to the user: if successful, it shows the shortened URL as a clickable link; otherwise, it displays an error message.  
2) home.html: Contains html code for the home page of the web application
3) style.css: Contains CSS code for the homepage
##Intial Step to compile and run this demo:
**Warning**: Ensure your local host is not mapped to anything or `us.ex` is not already mapped in your `hosts` file to avoid conflicts. Also, ensure you can safely map your local host
Open a terminal and run the following command to map `localhost` to `us.ex`:
For Unix-based systems (Linux/macOS):
```
sudo bash -c 'echo "127.0.0.1 us.ex" >> /etc/hosts'
```
For Windows:
```
Add-Content -Path C:\Windows\System32\drivers\etc\hosts -Value "127.0.0.1 us.ex"
```
##Second Step:
Intall docker. For more information on Docker and to download it, visit the [Docker Get Started page](https://www.docker.com/get-started/).

##Third Step:
Clone the MAKE_SHORT_URL repository and run the following commands:
```
git clone https://github.com/DirtyVoid/MAKE_SHORT_URL.git
cd MAKE_SHORT_URL
docker build --no-cache -t my-url-shortener .
docker run -p 80:80 my-url-shortener
```
##Fourth Step:
Open a browser and enjoy shortening URL. URLs are only valid for 1 day but you can change TTL varible in redis_conn.rs to change its behavior
