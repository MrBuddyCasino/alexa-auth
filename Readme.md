Alexa Authentication Server
===========================

This is a small web server that belongs to the https://github.com/MrBuddyCasino/ESP32_Alexa project and handles authentication.


## Required Software

Current version of Rust, ideally installed using rustup, and Cargo.
* `rustup self update`
* `rustup update`

## Build

Execute `cargo build --release`. The binary will be at `target/release/alexa-auth`.


## Run

You need to provide the following environment variables:
* client_id
* client_secret
* redirect_uri

If they are missing, the program will exit with an error.
I run it as a service on Ubuntu behind nginx, this is my
SystemD unit file:

```
[Unit]
Description=Alexa Auth Service
After=network.target

[Service]
Environment=client_id=amzn1.application-oa2-client.xxxxxxxxxxxxxxxxxx
Environment=client_secret=xxxxxxxxxxxxxxxxx
Environment=redirect_uri=https://alexa.boeckling.net/return
Type=simple
User=www-data
ExecStart=/home/michaelboeckling/alexa-auth/target/release/alexa-auth
WorkingDirectory=/home/michaelboeckling/alexa-auth
[Install]
WantedBy=multi-user.target
```

I did not manage to cross-compile it from my Mac due to OpenSSL.