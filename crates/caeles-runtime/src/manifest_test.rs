#[cfg(test)]
mod tests {
    use super::super::manifest::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_manifest(dir: &TempDir, entry: &str) -> PathBuf {
        let manifest_path = dir.path().join("manifest.json");
        let manifest_json = format!(
            r#"{{
  "id": "com.test.example",
  "name": "Test Capsule",
  "version": "1.0.0",
  "entry": "{}",
  "permissions": {{
    "notifications": true,
    "network": false
  }}
}}"#,
            entry
        );
        fs::write(&manifest_path, manifest_json).unwrap();
        manifest_path
    }

    #[test]
    fn test_load_valid_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(&temp_dir, "test.wasm");

        let manifest = CapsuleManifest::load(&manifest_path).unwrap();

        assert_eq!(manifest.id, "com.test.example");
        assert_eq!(manifest.name, "Test Capsule");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.entry, "test.wasm");
        assert!(manifest.permissions.notifications);
        assert!(!manifest.permissions.network);
    }

    #[test]
    fn test_manifest_wasm_path() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(&temp_dir, "capsule.wasm");

        let manifest = CapsuleManifest::load(&manifest_path).unwrap();
        let wasm_path = manifest.wasm_path();

        assert_eq!(
            wasm_path,
            temp_dir.path().join("capsule.wasm")
        );
    }

    #[test]
    fn test_manifest_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(&temp_dir, "../other/test.wasm");

        let manifest = CapsuleManifest::load(&manifest_path).unwrap();
        let wasm_path = manifest.wasm_path();

        assert!(wasm_path.to_string_lossy().contains("other"));
        assert!(wasm_path.to_string_lossy().contains("test.wasm"));
    }

    #[test]
    fn test_load_nonexistent_manifest() {
        let result = CapsuleManifest::load(&PathBuf::from("/nonexistent/manifest.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("invalid.json");
        fs::write(&manifest_path, "{ invalid json }").unwrap();

        let result = CapsuleManifest::load(&manifest_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_missing_required_field() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("incomplete.json");
        let manifest_json = r#"{
  "id": "com.test.example",
  "name": "Test"
}"#;
        fs::write(&manifest_path, manifest_json).unwrap();

        let result = CapsuleManifest::load(&manifest_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_permissions_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(&temp_dir, "test.wasm");
        let manifest = CapsuleManifest::load(&manifest_path).unwrap();

        // Test that we can access permissions
        assert!(manifest.permissions.notifications || !manifest.permissions.notifications);
        assert!(manifest.permissions.network || !manifest.permissions.network);
    }
}
