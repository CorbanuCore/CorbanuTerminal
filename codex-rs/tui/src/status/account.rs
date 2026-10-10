#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StatusAccountDisplay {
    ChatGpt {
        email: Option<String>,
        plan: Option<String>,
    },
    ApiKey {
        /// The environment variable the key is read from, such as
        /// `OPENAI_API_KEY`; `None` for a saved key.
        env_var: Option<String>,
    },
}
