// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod error;
mod gateway;
mod signer;
mod smart_account;

use dioxus::prelude::*;
use signer::Multisig;
use solana_sdk::{signature::Keypair, signer::Signer};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Hero {}
    }
}

#[component]
fn SignerCmp() -> Element {
    let mut signer: Signal<Option<Multisig>> = use_signal(|| None);
    let mut signer_string = use_signal(|| "no keypair".to_string());
    use_effect(move || {
        if let Some(multisig) = &*signer.read() {
            let vault = smart_account::vault(multisig);
            let out = format!("pubkey:{:?}-vault:{:?}", multisig.creator.pubkey(), vault);
            signer_string.set(out);
        }
    });
    let _ = use_resource(move || async move {
        if let Some(multisig) = &*signer.read() {
            match smart_account::get_or_create(multisig).await {
                Ok(multisig) => {
                    println!("{:?}", multisig.create_key);
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
        Ok::<(), crate::error::Error>(())
    });
    let _ = use_resource(move || async move {
        if let Some(multisig) = &*signer.read() {
            match smart_account::dummy(multisig).await {
                Ok(_) => {
                    println!("ok ok");
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
    });
    rsx! {
        div {
            button {
                onclick: move |_| {
                    if let Ok(keypair) = crate::signer::get_or_set() {
                        signer.set(Some(keypair))
                    }
                },
                "click me"
            }
        }
        div { "{signer_string}" }
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div { id: "hero",
            img { src: HEADER_SVG, id: "header" }
            SignerCmp {}
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus",
                    "💫 VSCode Extension"
                }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}
