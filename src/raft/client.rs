pub struct Client {
    pub address: String,
    pub node_id: String,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Node {}: ({})]", self.node_id, self.address)
    }
}
