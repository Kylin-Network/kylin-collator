use jsonrpc_core::*;

#[derive(Clone, Default)]
struct Meta(usize);
impl Metadata for Meta {}

pub fn main() {
	let mut io = MetaIoHandler::default();

	io.add_method_with_meta("say_hello", |_params: Params, meta: Meta| async move {
		Ok(Value::String(format!("Hello World: {}", meta.0)))
	});

	let request = r#"{"jsonrpc": "2.0", "method": "say_hello", "params": [42, 23], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"Hello World: 5","id":1}"#;

	let headers = 5;
	assert_eq!(
		io.handle_request_sync(request, Meta(headers)),
		Some(response.to_owned())
	);
}
