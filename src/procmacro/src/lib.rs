extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro::{TokenStream};
use syn::{Lit, MetaItem};

#[proc_macro_derive(Handler, attributes(From))]
pub fn impl_handler(ts: TokenStream) -> TokenStream {
	let raw = ts.to_string();
	let ast = syn::parse_derive_input(&raw).unwrap();
	let generics = ast.generics;
	if !generics.lifetimes.is_empty() || !generics.ty_params.is_empty() {
		panic!("item can not be generic typed currently");
	}

	let mut from: Option<String> = None;
	for attr in ast.attrs {
		if attr.name() == "From" {
			match attr.value {
				MetaItem::NameValue(_, l) => {
					match l {
						Lit::Str(v, _) => {
							from = Some(v);
						}
						_ => {}
					}
				}
				_ => {}
			}
		}
	}

	let base_type: String;
	let from = from.unwrap().to_lowercase();
	match from.as_str() {
		"fs::readable" => {
			base_type = "spk::h2tp::fs::read::Readable".to_string();
		}
		"router" => {
			base_type = "spk::h2tp::router::Router".to_string();
		}
		_ => {
			panic!("{}", format!("unknown from `{}`", from))
		}
	}

	let name = ast.ident;
	let base_type = syn::Ident::new(base_type);
	let code = quote::quote!(
		#[async_trait]
		impl spk::h2tp::handler::Handler for #name {
			#[inline]
			async fn handle<'a, 'c, 'h: 'a>(
				&'h self,
				req: &'a mut spk::h2tp::request::Request<'c>,
				resp: &'a mut spk::h2tp::response::Response<'c>,
			) -> () {
				#base_type::handle(self, req, resp).await
			}
		}
	);
	return code.parse().unwrap();
}