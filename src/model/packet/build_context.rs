extern crate path_clean;

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use tokio::fs;

use self::path_clean::PathClean;
use crate::model::packet::Base64Encoded;

/// Build context. Basically a list of files, base64 encoded
///
/// This blob may be very big due to the nature of files.
/// The external actor is recommended to store this context on S3 or other object storage services.
#[derive(Serialize, Deserialize)]
pub struct BuildContext {
    files: HashMap<String, Base64Encoded>,
}

impl BuildContext {
    /// Extract the entire build context into the target path
    pub async fn extract_into(&self, target_path: &Path) -> Result<bool, Box<dyn Error>> {
        for (location, bytes) in &self.files {
            let dest = target_path.join(location).clean();
            if !dest.starts_with(target_path) {
                return Ok(false);
            }

            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dest)
                .await?;

            bytes.write_to(&mut file).await?;
        }

        Ok(true)
    }
}
