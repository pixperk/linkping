use dotenvy::dotenv;
use std::env;

pub struct Config{
    pub port : u16,
    pub db_url: String,
}

impl Config{
    pub fn new() -> Self {
        dotenv().ok();
        let port = env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().unwrap();
        let db_url = env::var("DATABASE_URL").unwrap();
        
        Self { port, db_url }
          }
}