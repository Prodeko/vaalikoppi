use envconfig::Envconfig;
use validator::Validate;

#[derive(Envconfig, Validate)]
pub struct Config {
    #[envconfig(from = "DATABASE_URL")]
    #[validate(length(min = 1, max = 1024))]
    pub database_url: String,
}
