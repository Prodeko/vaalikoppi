use crate::models::Token;

#[derive(Clone, Debug)]
pub struct Ctx {
    is_admin: bool,
    token: Option<Token>,
}

impl Ctx {
    pub fn new(is_admin: bool, token: Option<Token>) -> Self {
        Self { is_admin, token }
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    pub fn token(&self) -> Option<Token> {
        self.token.clone()
    }
}
