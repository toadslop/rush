use std::ops::{Deref, DerefMut};

use fake::{
    faker::{
        company::en::CompanyName,
        internet::en::{Password, SafeEmail},
    },
    Dummy, Fake, Faker,
};
use rand::Rng;
use rush_data_server::model::{
    account::{Account, CreateAccountDto},
    email_address::EmailAddress,
    Table,
};
use surrealdb::opt::RecordId;

#[derive(Debug, Clone)]
pub struct DummyAccountDto(CreateAccountDto);

impl Dummy<Faker> for DummyAccountDto {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        Self(CreateAccountDto {
            email: SafeEmail().fake_with_rng(rng),
            name: CompanyName().fake_with_rng::<String, R>(rng),
            password: Password(8..16).fake_with_rng(rng),
        })
    }
}

impl Deref for DummyAccountDto {
    type Target = CreateAccountDto;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DummyAccountDto {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct DummyAccount(Account);
impl Dummy<Faker> for DummyAccount {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let email = SafeEmail().fake_with_rng::<String, R>(rng);
        Self(Account {
            id: Some(RecordId::from((Account::name(), email.as_ref()))),
            email: Some(EmailAddress(email)),
            name: Some(CompanyName().fake_with_rng(rng)),
            confirmed: Some(false.fake_with_rng(rng)),
            instances: Some(vec![]),
        })
    }
}

impl Deref for DummyAccount {
    type Target = Account;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
