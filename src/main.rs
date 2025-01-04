mod error;
mod gateway;
mod signer;
mod smart_account;

use dioxus::prelude::*;
use signer::PrettyKeypair;
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
    let mut signer: Signal<Option<PrettyKeypair>> = use_signal(|| None);
    let mut signer_string = use_signal(|| "no keypair".to_string());
    use_effect(move || {
        if let Some(signer) = &*signer.read() {
            signer_string.set(signer.0.pubkey().to_string());
        }
    });
    use_effect(move || {
        if let Ok(keypair) = crate::signer::get_or_set() {
            let keypair_ = keypair.0.insecure_clone();
            signer.set(Some(keypair));
            spawn(async move {
                let res = smart_account::create_smart_account(keypair_).await;
                println!("{:?}", res);
            });
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
