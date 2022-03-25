use chrono::Local;
use reqwest::blocking::Client;

use cardamom_lib::{
    build_patch,
    card::{Card, Cards},
    card_repository::CardRepository,
    carddav::addressbook_path,
    error::*,
    remote_card_repository::RemoteCardRepository,
};

/// Tests the remote card repository methods by running a simple flow create -> read -> update ->
/// delete.
#[test]
fn test_remote_card_repository() -> Result<()> {
    let host = "http://localhost:5232";
    let client = Client::new();
    let repository = RemoteCardRepository::new(host, &client)?;

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

    // Creates a card and checks that the etag is well set.
    repository.insert(&mut card)?;
    assert!(!card.etag.is_empty());

    // Checks that the card has been created.
    let expected_card = repository.select(id)?;
    assert_eq!(expected_card.id, card.id);
    assert_eq!(expected_card.etag, card.etag);
    assert_eq!(expected_card.content, card.content);

    // Updates a card and checks that the etag is well changed.
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
    repository.update(&mut card)?;
    assert_ne!(expected_card.etag, card.etag);

    // Checks that the card has been updated.
    let expected_card = repository.select(id)?;
    assert_eq!(expected_card.id, card.id);
    assert_eq!(expected_card.etag, card.etag);
    assert_eq!(expected_card.content, card.content);

    // Deletes the card.
    repository.delete(&card)?;

    // Checks that the card has been deleted.
    let res = repository.select(id);
    assert!(matches!(
        res.unwrap_err(),
        CardamomError::ReadCardError(_, _)
    ));

    Ok(())
}

#[test]
fn test_build_patch() {
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
    repository.insert(&mut card).unwrap();

    let remote = repository.select_all().unwrap();
    let local = Cards::default();
    let cache = Cards::default();

    let patch = build_patch(local, cache, remote);
    assert_eq!(patch, vec![]);
}
