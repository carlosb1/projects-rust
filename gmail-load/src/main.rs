extern crate imap;

use html2text::from_read;
use mailparse::parse_mail;
use std::env;

fn fetch_inbox_top() -> imap::error::Result<String> {
    let domain = "imap.gmail.com";
    let password = env::var("GMAIL_PASS").unwrap();
    let username = env::var("GMAIL_USER").unwrap();

    println!("username={:?} password={:?}", username, password);

    let tls = native_tls::TlsConnector::builder().build().unwrap();

    let client = imap::connect((domain, 993), domain, &tls).unwrap();

    let mut imap_session = client.login(username, password).map_err(|e| e.0).unwrap();
    println!("messages");

    imap_session.select("INBOX")?;

    //list messages
    let messages = imap_session.fetch("1", "RFC822")?;
    //let messages = imap_session.fetch("1", "BODY[TEXT]")?;

    // get message

    for message in messages.iter() {
        let body = message.body().expect("message did not have a body!");
        let body = std::str::from_utf8(body)
            .expect("message was not valid utf-8")
            .to_string();
        let parsed = parse_mail(body.as_bytes()).unwrap();
        let txt = parsed.subparts[0].get_body().unwrap();
        let html_parsed = from_read(txt.as_bytes(), 20);
        println!(" - {:?}", html_parsed.clone());
    }
    imap_session.logout()?;

    Ok("Ok".to_string())
}

fn main() {
    let _res = fetch_inbox_top();
    println!("Hello, world!");
}
