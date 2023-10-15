#[derive(Clone, Debug)]
pub struct Ctx {
    is_admin: bool,
}

impl Ctx {
    pub fn new(is_admin: bool) -> Self {
        Self { is_admin }
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
}
