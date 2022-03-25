use crate::{
    card::{Card, Cards},
    error::*,
};

pub trait CardRepository {
    fn insert(&self, card: &mut Card) -> Result<()>;
    fn select(&self, id: &str) -> Result<Card>;
    fn select_all(&self) -> Result<Cards>;
    fn update(&self, card: &mut Card) -> Result<()>;
    fn delete(&self, card: &Card) -> Result<()>;
}
