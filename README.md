# MAKE_SHORT_URL

## Description
MAKE_SHORT_URL is a web application for creating short URLs, similar to TinyURL, using `actix_web` with `redis` on Docker.

### Source Files
- `base62.rs`: Base62 encoding logic.
- `redis_conn.rs`: Redis interaction, setup, and event handling.
- `server.rs`: Main application entry, server setup.
- `test.rs`: Project tests.

### Static Files
- `handle_button.js`: Event listener for form submission and URL shortening logic.
- `home.html`: Homepage HTML.
- `style.css`: Homepage styling.

### Setup
1. **Localhost Mapping**: Ensure no conflicts in `/etc/hosts` (Unix) or `C:\Windows\System32\drivers\etc\hosts` (Windows) and map `us.ex` to `127.0.0.1`.
2. **Docker**: Install Docker ([Get Started with Docker](https://www.docker.com/get-started/)).
3. **Clone and Run**:
   ```bash
   git clone https://github.com/DirtyVoid/MAKE_SHORT_URL.git
   cd MAKE_SHORT_URL
   docker build --no-cache -t my-url-shortener .
   docker run -p 80:80 my-url-shortener
