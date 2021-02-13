use std::error::Error;
use std::path::Path;

use crate::builder::execute_build;

pub(crate) async fn spawn(target_path: &Path) -> Result<bool, Box<dyn Error>> {
	let path = target_path.to_owned();
	Ok(tokio::spawn(async move {
		// Run cleanup utility

		// This occurs due to io/process errors,
		// in that case the only appropriate solution is to panic and let k8s
		// restart the pod
		execute_build(&path).unwrap()

		// Run packaging utility
	}).await?)
}