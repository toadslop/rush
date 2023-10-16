pub mod account;
pub mod email_address;
pub mod instance;

pub trait Table {
    fn name() -> &'static str;
}

pub trait CreateTable {
    fn id(&self) -> &str;
}
