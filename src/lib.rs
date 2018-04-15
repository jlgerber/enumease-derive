#![recursion_limit="128"]
// proc_macro is a support crate which provides proc_macro_derive, as well as TokenStream
// It is speicifcally designed to make creating procedural macros simpler.
extern crate proc_macro;
extern crate proc_macro2;
// syn provides a parser of the rust language.
extern crate syn;
// quote provides a convenience macro (quote!) to
// turn Rust syntax tree data structures into tokens of source code
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{ Fields::Unit };

#[derive(Debug)]
enum MatchType {
   Unit(syn::Ident),
   NonUnit(syn::Ident),
}

enum Mode {
    Iter,
    FromStr,
    Display,
}

#[proc_macro_derive(EnumDisplay)]
/// EnumDisplay procedural macro generates an implementation of
/// the std::fmt::Display trait for simple enums.
pub fn enum_display(input: TokenStream) -> TokenStream {

    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_enum_ease(ast, Mode::Display);

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(EnumIter)]
/// EnumIter generates an implementation of the EnumIter trait
/// for simple enums. The EnumIter trait is defined thusly:
///
/// ```rust
/// pub trait EnumIter {
///     fn iterator() -> Iter<'static, Self> where Self: marker::Sized;
/// }
/// ```
///
pub fn enum_iter(input: TokenStream) -> TokenStream {

    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_enum_ease(ast, Mode::Iter);

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(EnumFromStr)]
/// The EnumFromStr generates an implementation of the EnumFromStr
/// trait, which allows one to generate an enum value from the
/// analogous string.
///
///  The trait is defined like so:
///
/// ```
/// pub trait EnumFromStr: EnumIter {
///     fn from_str(key: &str) -> Option<Self> where Self: marker::Sized;
/// }
///```
///
pub fn enum_from_str(input: TokenStream) -> TokenStream {

    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_enum_ease(ast, Mode::FromStr);

    // Return the generated impl
    gen.into()
}

// Implementation for generating all traits. We pass a mode enum to
// indicate which trait to generate. Since the body is largely the same,
// we can reuse 90% of the code.
fn impl_enum_ease(ast: syn::DeriveInput, mode: Mode) -> quote::Tokens {
    let name = &ast.ident;
    // upper case version of the name
    let name_upper =  syn::parse_str::<syn::Ident>(&format!("{}", name.as_ref()).to_uppercase()).unwrap();

    // match against the target struct and build up a vector of fields
    let  match_types: Vec<MatchType> = match ast.data {
        syn::Data::Enum(vdata) => {
            match vdata {
                syn::DataEnum {enum_token:_,brace_token:_, variants:punctuated } => {
                    let mut idents = Vec::new();
                    for p in punctuated {
                        match p {
                            syn::Variant {attrs:_, ident , fields, discriminant: _} => {
                                if fields == Unit {
                                    idents.push(MatchType::Unit(ident));
                                } else {
                                    idents.push(MatchType::NonUnit(ident));
                                }
                            }
                        }
                    }
                    idents
                },
            }
        },
        syn::Data::Struct(_) => panic!("You can only derive this on enums!"),
        syn::Data::Union(_) => panic!("You can only derive this on enums!"),
    };

    let array_size = match_types.len();
    let mut enum_vars = quote::Tokens::new();
    let mut match_branches = quote::Tokens::new();

    for wrapped_ident in match_types {
        let ident;
        let branchvar = match wrapped_ident {
            MatchType::Unit(identv) => {
                ident = identv;
                format!("{}::{}", name.to_string(), ident.to_string())
            },
            MatchType::NonUnit(_identv) => {
                // this doesnt really work for non simple enums (ie enums
                // whose values are constructed with arguments )
                panic!("Does not support NonUnit Enum Values");
                //ident = identv;
                //format!("{}::{}", name.to_string(), ident.to_string())
            },
        };

        let enum_var = syn::parse_str::<syn::PatPath>(&branchvar).unwrap();
        enum_vars.append_all(quote!(#enum_var,));

        let ident_str = ident.to_string();
        let match_branch = quote!( #enum_var => write!(f, #ident_str), );
        match_branches.append_all(match_branch);
    }

    match mode {
        Mode::Iter => {
            quote!{
                impl EnumIter for #name {
                    fn iterator() -> Iter<'static, Self> where Self: marker::Sized {
                        static #name_upper: [#name; #array_size] = [ #enum_vars ];
                        #name_upper.into_iter()
                    }
                }
            }
        },
        Mode::FromStr => {
            quote!{
                impl EnumFromStr for #name {
                    fn from_str(key: &str) -> Option<Self> where Self: marker::Sized {
                        for field in Self::iterator() {
                            let fieldstr = field.to_string();
                            if fieldstr == key {
                                return Some(field.clone());
                            }
                        }
                        None
                    }
                }

            }
        },
        Mode::Display => {
            quote!{
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        match *self {
                            #match_branches
                        }
                    }
                }
            }
        },
    }
}