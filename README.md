### Cinema

Cinema is a video sharing platform I built for my final year project. This is the API, built with Rust, Actix web and PostgreSQL it includes all the endpoints used by the frontend.

### Development setup
#### Requirements
- Docker
- docker-compose
- Cargo

#### Optional
- PgAdmin

#### Info
- PostgreSQL database running inside Docker
- API built with in Rust with Actix Web
- Diesel for database migrations

#### Setup
1. Install Diesel CLI `cargo install diesel_cli`
2. Start docker `docker-compose up -d`
3. Start actix web server `cargo run`
4. Perform database migrations `diesel migration run`
5. Ready to go
