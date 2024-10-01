use prometheus_client::encoding::EncodeLabelSet;

#[derive(Debug, Clone, Hash, Eq, PartialEq, EncodeLabelSet)]
pub struct DatabaseQuery {
	pub query: String,
}
