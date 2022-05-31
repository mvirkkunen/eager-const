extern crate proc_macro;
use proc_macro::TokenStream;

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::str;
use syn::{
    parse_macro_input, Expr, Ident, Token, Type, Visibility,
    parse::{Parse, ParseStream, Result},
};
use quote::quote;

struct EagerConsts(Vec<EagerConst>);

struct EagerConst {
    vis: Visibility,
    name: Ident,
    ty: Type,
    init: Expr,
}

impl Parse for EagerConsts {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut result = Vec::new();

        while !input.is_empty() {
            let vis: Visibility = input.parse()?;
            input.parse::<Token![const]>()?;
            let name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            let ty: Type = input.parse()?;
            input.parse::<Token![=]>()?;
            let init: Expr = input.parse()?;
            input.parse::<Token![;]>()?;

            result.push(EagerConst {
                vis,
                name,
                ty,
                init,
            });
        }

        Ok(EagerConsts(result))
    }
}

const SELF_PATH: &str = env!("CARGO_MANIFEST_DIR");

#[proc_macro]
pub fn eager_const(input: TokenStream) -> TokenStream {
    if cfg!(feature = "inside") {
        return quote!().into();
    }

    let consts = parse_macro_input!(input as EagerConsts).0;

    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("no CARGO_MANIFEST_DIR");
    let tmp_dir = Path::new("/tmp/.tmp-crate/");

    fs::remove_dir_all(tmp_dir).ok();

    fs::create_dir_all(tmp_dir).expect("create_dir_all failed");

    copy_crate(crate_dir, tmp_dir).expect("copy_crate failed");

    modify_manifest(tmp_dir).expect("modify_manifest failed");

    modify_main(tmp_dir.join("src/main.rs"), &consts).expect("modify_main failed");

    let cargo_path = env::var("CARGO").expect("no CARGO");

    remove_cargo_env(Command::new(&cargo_path))
        .current_dir(tmp_dir)
        .arg("build")
        .spawn()
        .expect("build spawn failed")
        .wait()
        .expect("build wait failed");

    let output = remove_cargo_env(Command::new(&cargo_path))
        .current_dir(tmp_dir)
        .arg("run")
        .arg("--quiet")
        .output()
        .expect("run failed");

    if !output.status.success() {
        panic!("\nError: {}", String::from_utf8_lossy(&output.stderr));
    }

    let values = str::from_utf8(&output.stdout).expect("output is not utf-8").lines().collect::<Vec<_>>();

    let mut items = Vec::new();

    for (i, c) in consts.iter().enumerate() {
        let EagerConst { vis, name, ty, .. } = &c;
        let value: syn::Expr = syn::parse_str(values[i]).expect("syn::parse_str failed");

        items.push(quote! {
            #vis const #name: #ty = #value;
        });
    }

    (quote! {
        #(#items)*
    }).into()
}

fn remove_cargo_env(mut cmd: Command) -> Command {
    for (key, _) in env::vars() {
        if key.starts_with("CARGO_") || key == "CARGO" {
            cmd.env_remove(key);
        }
    }

    cmd
}

fn copy_crate<F, T>(from: F, to: T) -> std::io::Result<()>
where
    F: AsRef<Path>, T: AsRef<Path>
{
    let from = from.as_ref();
    let to = to.as_ref();

    eprintln!("{:?} {:?}", from, to);
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
            if name == "target" || name == ".git" {
                continue;
            }

            let target = to.join(name);

            if path.is_dir() {
                fs::create_dir(&target)?;
                copy_crate(&path, &target)?;
            } else if path.is_file() {
                fs::copy(&path, &target).map(|_| ())?;
            }
        }
    }

    Ok(())
}

fn modify_manifest<P>(path: P) -> std::io::Result<()>
where
    P: AsRef<Path>
{
    use toml::value::*;

    let manifest_path = path.as_ref().join("Cargo.toml");

    let mut manifest: Value = fs::read_to_string(&manifest_path)?.parse().expect("manifest read_to_String failed");

    let root = manifest.as_table_mut().expect("manifest is not a table");

    let deps = root
        .entry("dependencies")
        .or_insert_with(|| Value::Table(Table::new()))
        .as_table_mut()
        .expect("dependencies is not a table");

    for (name, value) in deps.iter_mut() {
        if name == "eager-const" {
            let mut dep = Table::new();
            dep.insert("path".into(), SELF_PATH.into());
            dep.insert("features".into(), vec!["inside".to_string()].into());
            *value = dep.into();
        } else if let Some(dep) = value.as_table_mut() {
            if let Some(path) = dep.get_mut("path") {
                *path = manifest_path.join(path.as_str().expect("path is not a string")).canonicalize().expect("canonicalize path failed").to_str().expect("dependency path is not a valid string").into();
            }
        }
    }

    let mut dep = Table::new();
    dep.insert("path".into(), Path::new(SELF_PATH).join("../serde-rust").canonicalize().expect("canonicalize serde-rust failed").to_str().expect("serde-rust path is not a valid string").into());
    deps.insert("serde-rust".into(), dep.into());

    fs::write(manifest_path, toml::to_string(&manifest).expect("toml::to_string failed")).expect("manifest write failed");

    Ok(())
}

fn modify_main<P>(path: P, consts: &Vec<EagerConst>) -> std::io::Result<()>
where
    P: AsRef<Path>
{
    use syn::*;

    let path = path.as_ref();

    let File { shebang, attrs, mut items } = parse_file(&fs::read_to_string(&path)?).expect("parse_file main failed");

    items.retain(|item| {
        if let Item::Fn(fnitem) = item {
            fnitem.sig.ident != "main"
        } else {
            true
        }
    });

    let mut values = Vec::new();

    for c in consts.iter() {
        let init = &c.init;

        values.push(quote! {
            println!("{}", serde_rust::to_string(&#init).expect("serde_rust failed"));
        });
    }

    let new_main = quote! {
        #shebang
        #(#attrs)*
        #(#items)*

        fn main() {
            #(#values)*
        }
    }.to_string();

    fs::write(&path, new_main)
}