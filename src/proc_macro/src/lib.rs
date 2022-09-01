extern crate proc_macro;

use proc_macro::TokenStream;

fn get_type_name(ts: TokenStream) -> String {
	let mut status = 0;
	let mut buf = String::new();
	for tree in ts {
		match status {
			0 => {
				if tree.to_string() == "struct" {
					status += 1;
				}
			}
			1 => {
				buf.push_str(tree.to_string().as_str());
				status += 1;
			}
			2 => {
				if tree.to_string() == "<" {
					buf.push_str("<");
					status += 1;
				} else {
					return buf;
				}
			}
			3 => {
				if tree.to_string() == ">" {
					buf.push_str(">");
					return buf;
				}
				buf.push_str(tree.to_string().as_str());
			}
			_ => {}
		}
	}
	return buf;
}

#[proc_macro_derive(Handler)]
pub fn impl_handler(ts: TokenStream) -> TokenStream {
	let name = get_type_name(ts);
	if name.contains("<") {
		panic!("item can not be generice typed and must has empty lifetime params");
	}
	return format!("#[async_trait]\
	impl crate::h2tp::handler::Handler for {} {{\
		#[inline]\
		async fn handle<'a, 'c, 'h: 'a>(\
			&'h self,\
			req: &'a mut crate::h2tp::request::Request<'c>,\
			resp: &'a mut crate::h2tp::response::Response<'c>\
		) -> () {{\
			crate::h2tp::fs::read::Readable::handle(self, req, resp).await\
		}}\
	}}", name).parse().unwrap();
}