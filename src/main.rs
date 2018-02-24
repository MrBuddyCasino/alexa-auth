#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate hyper;
extern crate reqwest;
extern crate url;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rocket_contrib;

use std::error::Error;
use std::env;
use std::io;
use std::io::Read;
use std::collections::HashMap;
use url::Url;
use rocket::response::NamedFile;
use rocket::response::content;
use rocket::request::Form;
use rocket_contrib::Template;


#[derive(Debug, FromForm)]
struct FormInput {
    mac: String
}


/// show form to enter MAC
#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

/// post MAC, get refresh token
#[post("/add_device", data = "<form_data>")]
fn add_device<'r>(form_data: Result<Form<FormInput>, Option<String>>) -> content::Html<String> {
    let client_id = env::var("client_id").unwrap();
    let redirect_uri = env::var("redirect_uri").unwrap();

    match form_data {
        Ok(form) => {
            let url_str = format!("https://www.amazon.com/ap/oa?\
                client_id={}\
                &scope=alexa:all\
                &scope_data={{\"alexa:all\":{{\"productID\":\"alexa_dev\",\"productInstanceAttributes\":{{\"deviceSerialNumber\":{:?}}}}}}}\
                &response_type=code\
                &redirect_uri={}", client_id, form.get().mac, redirect_uri);

            let url = Url::parse(url_str.as_str()).unwrap();

            let client = reqwest::Client::new();
            let res = client.get(url).send();

            let mut resp_body = String::new();
            res.unwrap().read_to_string(&mut resp_body).unwrap();
            println!("{}", resp_body);

            content::Html(resp_body)
        }
        Err(Some(f)) => content::Html(format!("Invalid form input: {}", f)),
        Err(None) => content::Html(format!("Form input was invalid UTF8.")),
    }
}


/// post refresh token, get new auth token
#[get("/auth/refresh/<refresh_token>")]
fn get_new_token(refresh_token: String) -> Result<String, String> {
    println!("refresh_token_param: {}", refresh_token);

    let client_id = env::var("client_id").unwrap();
    let client_secret = env::var("client_secret").unwrap();

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.as_str()),
        ("client_id", &client_id),
        ("client_secret", &client_secret)];

    let client = reqwest::Client::new();
    let res = client.post("https://api.amazon.com/auth/o2/token")
        .form(&params)
        .send();

    let mut resp_body = String::new();
    res.unwrap().read_to_string(&mut resp_body).unwrap();
    //println!("{}", resp_body);

    Ok(resp_body)
}

#[get("/consent")]
fn get_consent() -> io::Result<NamedFile> {
    NamedFile::open("static/consent.html")
}

#[derive(FromForm)]
struct ReturnParams { code: String, scope: String }

#[get("/return?<return_params>")]
fn get_return(return_params: ReturnParams) -> Template {
    let code = &return_params.code;
    println!("code: {}", code);

    let client_id = env::var("client_id").unwrap();
    let client_secret = env::var("client_secret").unwrap();
    let redirect_uri = env::var("redirect_uri").unwrap();

    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("redirect_uri", &redirect_uri)
    ];

    let client = reqwest::Client::new();
    let res = client.post("https://api.amazon.com/auth/o2/token")
        .form(&params)
        .send();

    let mut resp_body = String::new();
    res.unwrap().read_to_string(&mut resp_body).unwrap();
    println!("authorization_code result: {}", resp_body);

    // TODO: parse json from resp_body, extract access_token
    let v: serde_json::Value = serde_json::from_str(&resp_body.as_str()).unwrap();
    let access_token = &v["refresh_token"];

    let mut map = HashMap::new();
    map.insert("code", access_token.as_str());
    Template::render("show_code", &map)
}

fn main() {

    // check env vars
    env::var("client_id").map_err(|e|
        format!("{}{}", "client_id: ", e.description())).unwrap();

    env::var("client_secret").map_err(|e|
        format!("{}{}", "client_secret: ", e.description())).unwrap();

    env::var("redirect_uri").map_err(|e|
        format!("{}{}", "redirect_uri: ", e.description())).unwrap();

    rocket::ignite()
        .mount("/", routes![index, get_new_token, add_device, get_consent, get_return])
        .attach(Template::fairing())
        .launch();
}