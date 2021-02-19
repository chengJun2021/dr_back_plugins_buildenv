use crate::model::packet::Base64Encoded;

/// The result of the build.
///
/// Defaults to [`BuildStatus::LowLevelError`]
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum BuildStatus {
    /// A validator detected an error, human friendly description
    ValidationError(String),
    /// The webpack executable returned a non-success exit code.
    WebpackExit {
        /// Exit code, `0` should mean success and a non-zero exit code should be documented by webpack.
        code: i32,
        /// Webpack's logs
        #[serde(rename = "webpackOutputs")]
        webpack_outputs: WebpackOutputs,
    },
    /// A more primitive error, details will be emitted into the logs.
    LowLevelError,
    /// The build has succeeded.
    Success {
        /// Base-64 encoded zip file which can be inflated to find the build artefacts
        zip: Base64Encoded,
        /// Webpack's logs
        #[serde(rename = "webpackOutputs")]
        webpack_outputs: WebpackOutputs,
    },
}

impl Default for BuildStatus {
    fn default() -> Self {
        BuildStatus::LowLevelError
    }
}

/// Captured output streams of webpack
#[derive(Deserialize, Serialize)]
pub struct WebpackOutputs {
    /// Captured `stdout` of webpack. Can be displayed to the client.
    pub(crate) stdout: Base64Encoded,
    /// Captured `stderr` of webpack. Can be displayed to the client.
    pub(crate) stderr: Base64Encoded,
}
