use std::{ env, fs, path::Path };

use proc_macro2::TokenStream;
use quote::quote;
use serde::{ Deserialize, Serialize };
use syn::File;

const FALLBACK_JSON: &'static str = include_str!("fallback.json");
const SOURCE_URL: &'static str =
    "https://raw.githubusercontent.com/internetarchive/openlibrary/refs/heads/master/openlibrary/components/LibraryExplorer/ddc.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Class {
    Node {
        name: String,
        short: String,
        query: String,
        children: Vec<Box<Class>>,
        count: u64,
    },
    Leaf {
        name: String,
        short: String,
        query: String,
        count: u64,
    },
}

fn get_classes() -> Vec<Class> {
    if let Ok(response) = reqwest::blocking::get(SOURCE_URL).and_then(|r| r.error_for_status()) {
        if let Ok(result) = response.json::<Vec<Class>>() {
            return result;
        }
    }

    serde_json::from_str(FALLBACK_JSON).expect("Failed to deserialize fallback data.")
}

fn generate_class(output: &mut Vec<TokenStream>, class: Class) {
    match class {
        Class::Node { name, short, children, .. } => {
            let trimmed_code = short.trim_end_matches('X').to_string();
            if trimmed_code.len() > 4 {
                return;
            }
            output.push(
                quote! {
                {
                    let code: String = #trimmed_code.to_owned();
                    let _ = trie.insert(
                        code
                            .chars()
                            .map(|c| c.to_string().parse::<u8>().unwrap())
                            .collect::<Vec<_>>(),
                        Class {
                            code: code.clone(),
                            name: #name.to_owned(),
                            has_children: true,
                        }
                    );
                };
            }
            );

            for class in children {
                generate_class(output, *class);
            }
        }
        Class::Leaf { name, short, .. } => {
            let trimmed_code = short.trim_end_matches('X').to_string();
            if trimmed_code.len() > 4 {
                return;
            }
            output.push(
                quote! {
                {
                    let code: String = #trimmed_code.to_owned();
                    let _ = trie.insert(
                        code
                            .chars()
                            .map(|c| c.to_string().parse::<u8>().unwrap())
                            .collect::<Vec<_>>(),
                        Class {
                            code: code.clone(),
                            name: #name.to_owned(),
                            has_children: false,
                        }
                    );
                };
            }
            );
        }
    }
}

fn main() {
    let classes = get_classes();

    let mut class_items: Vec<TokenStream> = Vec::new();

    for class in classes {
        generate_class(&mut class_items, class);
    }

    let output =
        quote! {
        /// Representation of a single Dewey Decimal class
        #[derive(Clone, Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "specta", derive(specta::Type))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
        pub struct Class {
            /// DDC code of this class (ie `001`, `24`, etc)
            pub code: String,

            /// Friendly name of this class
            pub name: String,

            /// Whether this class has children
            pub has_children: bool
        }

        pub(crate) fn make_class_static() -> trie_rs::map::Trie<u8, Class> {
            let mut trie = trie_rs::map::TrieBuilder::new();

            #(#class_items)*

            trie.build()
        }
    };

    let str_out = prettyplease::unparse(&syn::parse2::<File>(output).unwrap());
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("classes.rs");
    fs::write(&dest_path, str_out).unwrap();

    println!("cargo::rerun-if-changed=fallback.json");
}
