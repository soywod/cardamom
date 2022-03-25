use chrono::Local;
use reqwest::blocking::Client;

use cardamom_lib::{
    card::Card, card_repository::CardRepository, error::*,
    remote_card_repository::RemoteCardRepository,
};

#[test]
fn test_remote_card_repository() {
    let host = "http://localhost:5232";
    let client = Client::new();
    let repository = RemoteCardRepository::new(host, &client).unwrap();

    let id = "4d60020b-7ee8-4a36-8d3a-eec1323def45";
    let mut card = Card {
        id: id.to_string(),
        etag: "".into(),
        date: Local::now(),
        content: [
            "BEGIN:VCARD",
            "VERSION:3.0",
            &format!("UID:{}", id),
            "EMAIL:test@mail.com",
            "FN:Test",
            "N:Nom;Prenom;;;",
            "ORG:Test",
            "TEL;TYPE=pref:06 06 06 06 06",
            "END:VCARD",
            "",
        ]
        .join("\r\n"),
    };

    // create a card and check that the etag is well set
    repository.insert(&mut card).unwrap();
    assert!(!card.etag.is_empty());

    // check that the card has been created
    let expected_card = repository.select(id).unwrap();
    assert_eq!(expected_card.id, card.id);
    assert_eq!(expected_card.etag, card.etag);
    assert_eq!(expected_card.content, card.content);

    // update a card and check that the etag is well changed
    card.content = [
        "BEGIN:VCARD",
        "VERSION:3.0",
        &format!("UID:{}", id),
        "EMAIL:test@mail.com",
        "FN:UpdatedTest",
        "N:Nom;Prenom;;;",
        "ORG:UpdatedTest",
        "TEL;TYPE=pref:06 06 06 06 06",
        "END:VCARD",
        "",
    ]
    .join("\r\n");
    repository.update(&mut card).unwrap();
    assert_ne!(expected_card.etag, card.etag);

    // check that the card has been updated
    let expected_card = repository.select(id).unwrap();
    assert_eq!(expected_card.id, card.id);
    assert_eq!(expected_card.etag, card.etag);
    assert_eq!(expected_card.content, card.content);

    // delete the card
    repository.delete(&card).unwrap();

    // check that the card has been deleted
    let res = repository.select(id);
    assert!(matches!(
        res.unwrap_err(),
        CardamomError::ReadCardError(_, _)
    ));
}
