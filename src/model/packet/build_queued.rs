#[derive(Serialize, Deserialize)]
pub struct BuildQueued {
    /// Whether the request is queued (`true`) or not (`false`)
    pub queued: bool,
}
