//! Document Manager for REST Client LSP Server
//!
//! This module provides thread-safe document state management using DashMap
//! for concurrent access without traditional locks.

use dashmap::DashMap;
use lsp_types::Url;
use std::sync::Arc;

/// Error types for document operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    /// Document not found in the manager
    NotFound,
    /// Invalid URI format or normalization failed
    InvalidUri(String),
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentError::NotFound => write!(f, "Document not found"),
            DocumentError::InvalidUri(msg) => write!(f, "Invalid URI: {}", msg),
        }
    }
}

impl std::error::Error for DocumentError {}

/// Thread-safe document manager for tracking open files
///
/// Uses DashMap for lock-free concurrent access, allowing multiple threads
/// to read and write document state without blocking.
#[derive(Debug, Clone)]
pub struct DocumentManager {
    /// Concurrent hash map storing document content by normalized URI
    documents: Arc<DashMap<String, String>>,
}

impl DocumentManager {
    /// Creates a new DocumentManager instance
    ///
    /// # Examples
    ///
    /// ```
    /// use rest_client::lsp_server::document::DocumentManager;
    ///
    /// let manager = DocumentManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            documents: Arc::new(DashMap::new()),
        }
    }

    /// Normalizes a URI to a consistent string format
    ///
    /// Handles file:// URIs and ensures consistent path representation
    /// across different platforms.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to normalize
    ///
    /// # Returns
    ///
    /// Returns the normalized URI string or an error if normalization fails
    fn normalize_uri(uri: &Url) -> Result<String, DocumentError> {
        // Convert to string and ensure consistent formatting
        let uri_str = uri.as_str();

        // Validate that it's a proper URI
        if uri_str.is_empty() {
            return Err(DocumentError::InvalidUri("Empty URI".to_string()));
        }

        // For file:// URIs, normalize the path
        if uri.scheme() == "file" {
            // Get the path and normalize it
            match uri.to_file_path() {
                Ok(path) => {
                    // Convert back to string with consistent separators
                    let normalized_path = path.to_string_lossy().replace('\\', "/");
                    Ok(format!("file://{}", normalized_path))
                }
                Err(_) => {
                    // If conversion fails, use the original URI
                    Ok(uri_str.to_string())
                }
            }
        } else {
            // For non-file URIs, use as-is
            Ok(uri_str.to_string())
        }
    }

    /// Inserts a new document into the manager
    ///
    /// If a document with the same URI already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the document
    /// * `content` - The document content
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `Err` if URI normalization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rest_client::lsp_server::document::DocumentManager;
    /// use lsp_types::Url;
    ///
    /// let manager = DocumentManager::new();
    /// let uri = Url::parse("file:///path/to/file.http").unwrap();
    /// manager.insert(uri, "GET https://example.com".to_string()).unwrap();
    /// ```
    pub fn insert(&self, uri: Url, content: String) -> Result<(), DocumentError> {
        let normalized_uri = Self::normalize_uri(&uri)?;
        self.documents.insert(normalized_uri, content);
        Ok(())
    }

    /// Updates an existing document's content
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the document to update
    /// * `content` - The new document content
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the document exists and was updated,
    /// `Err(DocumentError::NotFound)` if the document doesn't exist,
    /// or `Err(DocumentError::InvalidUri)` if URI normalization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rest_client::lsp_server::document::DocumentManager;
    /// use lsp_types::Url;
    ///
    /// let manager = DocumentManager::new();
    /// let uri = Url::parse("file:///path/to/file.http").unwrap();
    /// manager.insert(uri.clone(), "GET https://example.com".to_string()).unwrap();
    /// manager.update(uri, "POST https://example.com".to_string()).unwrap();
    /// ```
    pub fn update(&self, uri: Url, content: String) -> Result<(), DocumentError> {
        let normalized_uri = Self::normalize_uri(&uri)?;

        // Check if document exists before updating
        if self.documents.contains_key(&normalized_uri) {
            self.documents.insert(normalized_uri, content);
            Ok(())
        } else {
            Err(DocumentError::NotFound)
        }
    }

    /// Retrieves a document's content by URI
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the document to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` with the document content if found,
    /// or `None` if the document doesn't exist or URI normalization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rest_client::lsp_server::document::DocumentManager;
    /// use lsp_types::Url;
    ///
    /// let manager = DocumentManager::new();
    /// let uri = Url::parse("file:///path/to/file.http").unwrap();
    /// manager.insert(uri.clone(), "GET https://example.com".to_string()).unwrap();
    /// let content = manager.get(&uri);
    /// assert_eq!(content, Some("GET https://example.com".to_string()));
    /// ```
    pub fn get(&self, uri: &Url) -> Option<String> {
        let normalized_uri = Self::normalize_uri(uri).ok()?;
        self.documents
            .get(&normalized_uri)
            .map(|entry| entry.value().clone())
    }

    /// Removes a document from the manager
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the document to remove
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` with the document content if it was removed,
    /// or `None` if the document doesn't exist or URI normalization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rest_client::lsp_server::document::DocumentManager;
    /// use lsp_types::Url;
    ///
    /// let manager = DocumentManager::new();
    /// let uri = Url::parse("file:///path/to/file.http").unwrap();
    /// manager.insert(uri.clone(), "GET https://example.com".to_string()).unwrap();
    /// let removed = manager.remove(&uri);
    /// assert_eq!(removed, Some("GET https://example.com".to_string()));
    /// ```
    pub fn remove(&self, uri: &Url) -> Option<String> {
        let normalized_uri = Self::normalize_uri(uri).ok()?;
        self.documents
            .remove(&normalized_uri)
            .map(|(_, content)| content)
    }

    /// Returns the number of documents currently managed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rest_client::lsp_server::document::DocumentManager;
    /// use lsp_types::Url;
    ///
    /// let manager = DocumentManager::new();
    /// assert_eq!(manager.len(), 0);
    ///
    /// let uri = Url::parse("file:///path/to/file.http").unwrap();
    /// manager.insert(uri, "GET https://example.com".to_string()).unwrap();
    /// assert_eq!(manager.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Returns true if the manager has no documents
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rest_client::lsp_server::document::DocumentManager;
    ///
    /// let manager = DocumentManager::new();
    /// assert!(manager.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Clears all documents from the manager
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rest_client::lsp_server::document::DocumentManager;
    /// use lsp_types::Url;
    ///
    /// let manager = DocumentManager::new();
    /// let uri = Url::parse("file:///path/to/file.http").unwrap();
    /// manager.insert(uri, "GET https://example.com".to_string()).unwrap();
    /// assert_eq!(manager.len(), 1);
    ///
    /// manager.clear();
    /// assert_eq!(manager.len(), 0);
    /// ```
    pub fn clear(&self) {
        self.documents.clear();
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_new_manager_is_empty() {
        let manager = DocumentManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///test.http").unwrap();
        let content = "GET https://example.com".to_string();

        manager.insert(uri.clone(), content.clone()).unwrap();
        assert_eq!(manager.get(&uri), Some(content));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_insert_replaces_existing() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///test.http").unwrap();

        manager
            .insert(uri.clone(), "old content".to_string())
            .unwrap();
        manager
            .insert(uri.clone(), "new content".to_string())
            .unwrap();

        assert_eq!(manager.get(&uri), Some("new content".to_string()));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_update_existing_document() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///test.http").unwrap();

        manager.insert(uri.clone(), "original".to_string()).unwrap();
        let result = manager.update(uri.clone(), "updated".to_string());

        assert!(result.is_ok());
        assert_eq!(manager.get(&uri), Some("updated".to_string()));
    }

    #[test]
    fn test_update_nonexistent_document() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///nonexistent.http").unwrap();

        let result = manager.update(uri, "content".to_string());
        assert!(matches!(result, Err(DocumentError::NotFound)));
    }

    #[test]
    fn test_remove_existing_document() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///test.http").unwrap();
        let content = "GET https://example.com".to_string();

        manager.insert(uri.clone(), content.clone()).unwrap();
        let removed = manager.remove(&uri);

        assert_eq!(removed, Some(content));
        assert!(manager.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_document() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///nonexistent.http").unwrap();

        let removed = manager.remove(&uri);
        assert_eq!(removed, None);
    }

    #[test]
    fn test_clear() {
        let manager = DocumentManager::new();
        let uri1 = Url::parse("file:///test1.http").unwrap();
        let uri2 = Url::parse("file:///test2.http").unwrap();

        manager.insert(uri1, "content1".to_string()).unwrap();
        manager.insert(uri2, "content2".to_string()).unwrap();
        assert_eq!(manager.len(), 2);

        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_uri_normalization() {
        let manager = DocumentManager::new();
        let uri = Url::parse("file:///path/to/file.http").unwrap();

        manager.insert(uri.clone(), "content".to_string()).unwrap();

        // Should be able to retrieve with the same URI
        assert_eq!(manager.get(&uri), Some("content".to_string()));
    }

    #[test]
    fn test_concurrent_access() {
        let manager = Arc::new(DocumentManager::new());
        let mut handles = vec![];

        // Spawn multiple threads that insert documents concurrently
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let uri = Url::parse(&format!("file:///test{}.http", i)).unwrap();
                let content = format!("content {}", i);
                manager_clone.insert(uri, content).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all documents were inserted
        assert_eq!(manager.len(), 10);
    }

    #[test]
    fn test_concurrent_read_write() {
        let manager = Arc::new(DocumentManager::new());
        let uri = Url::parse("file:///shared.http").unwrap();

        // Insert initial document
        manager.insert(uri.clone(), "initial".to_string()).unwrap();

        let mut handles = vec![];

        // Spawn readers
        for _ in 0..5 {
            let manager_clone = Arc::clone(&manager);
            let uri_clone = uri.clone();
            let handle = thread::spawn(move || {
                let _content = manager_clone.get(&uri_clone);
            });
            handles.push(handle);
        }

        // Spawn writers
        for i in 0..5 {
            let manager_clone = Arc::clone(&manager);
            let uri_clone = uri.clone();
            let handle = thread::spawn(move || {
                let content = format!("update {}", i);
                let _ = manager_clone.update(uri_clone, content);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Document should still exist
        assert!(manager.get(&uri).is_some());
    }

    #[test]
    fn test_http_uri() {
        let manager = DocumentManager::new();
        let uri = Url::parse("http://example.com/test.http").unwrap();
        let content = "GET https://api.example.com".to_string();

        manager.insert(uri.clone(), content.clone()).unwrap();
        assert_eq!(manager.get(&uri), Some(content));
    }

    #[test]
    fn test_default() {
        let manager = DocumentManager::default();
        assert!(manager.is_empty());
    }
}
