//! 🔍 LANGUAGE DETECTION UTILITIES: Extracted via 3-strikes abstraction rule
//! WHY SEPARATE FILE: Language detection logic appears 3+ times across different contexts
//! - File extension mapping (developer.rs:88-104)
//! - Project type mapping (developer.rs:63-70)
//! - Content analysis patterns (developer.rs:74-84)
//!
//! AUDIT: Verify all language mappings are comprehensive and consistent

use std::collections::HashMap;

/// 🗺️ LANGUAGE MAPPING: Centralized file extension to language mapping
/// Inline reasoning: Single source of truth prevents inconsistent language detection
/// Alternative: Distributed mappings (rejected: maintenance burden, inconsistency risk)
pub fn language_from_extension(file_path: &str) -> String {
    match file_path.split('.').next_back().unwrap_or("") {
        "rs" => "rust".to_string(),
        "py" => "python".to_string(),
        "js" => "javascript".to_string(),
        "ts" => "typescript".to_string(),
        "go" => "go".to_string(),
        "java" => "java".to_string(),
        "cpp" | "cc" | "cxx" => "cpp".to_string(),
        "c" => "c".to_string(),
        "cs" => "csharp".to_string(),
        "rb" => "ruby".to_string(),
        "php" => "php".to_string(),
        "swift" => "swift".to_string(),
        "kt" => "kotlin".to_string(),
        "scala" => "scala".to_string(),
        // 🏗️ DEFAULT STRATEGY: Rust as default for Spiral Core ecosystem
        // Why: Primary language of this project, safest fallback for unknown extensions
        _ => "rust".to_string(),
    }
}

/// 🏷️ PROJECT TYPE DETECTION: Map project descriptors to languages
/// Inline reasoning: Enables language detection from repository/project context
pub fn language_from_project_type(project_type: &str) -> String {
    match project_type.to_lowercase().as_str() {
        "rust" | "cargo" => "rust".to_string(),
        "node" | "npm" | "yarn" => "javascript".to_string(),
        "python" | "pip" => "python".to_string(),
        "go" | "golang" => "go".to_string(),
        "java" | "maven" | "gradle" => "java".to_string(),
        // 🏗️ DEFAULT STRATEGY: Rust as fallback for unknown project types
        _ => "rust".to_string(),
    }
}

/// 📝 CONTENT ANALYSIS: Detect language from textual content patterns
/// Inline reasoning: Heuristic detection when file/project context unavailable
/// Audit: These patterns may need refinement based on detection accuracy
pub fn language_from_content(content: &str) -> String {
    let content_lower = content.to_lowercase();

    // 🎯 PRIORITY ORDER: More specific patterns first to avoid false positives
    if content_lower.contains("rust")
        || content_lower.contains("cargo")
        || content_lower.contains("use tokio")
    {
        "rust".to_string()
    } else if content_lower.contains("typescript") {
        "typescript".to_string()
    } else if content_lower.contains("javascript") {
        "javascript".to_string()
    } else if content_lower.contains("python")
        || content_lower.contains("import numpy")
        || content_lower.contains("from") && content_lower.contains("import")
    {
        "python".to_string()
    } else if content_lower.contains("golang") {
        "go".to_string()
    } else if content_lower.contains("go") {
        // Note: "go" is less specific, so checked after "golang"
        "go".to_string()
    } else {
        // 🏗️ DEFAULT STRATEGY: Rust as ultimate fallback
        "rust".to_string()
    }
}

/// 🔧 SMART LANGUAGE DETECTION: Multi-strategy detection with fallback chain
/// DECISION REASONING: Combines multiple detection methods for maximum accuracy
/// Priority: file_path → project_type → content_analysis → default
pub fn detect_language_from_context(
    file_path: Option<&str>,
    project_type: Option<&str>,
    content: &str,
) -> String {
    // 🥇 FIRST PRIORITY: File extension (most reliable indicator)
    if let Some(path) = file_path {
        let detected = language_from_extension(path);
        // Only use extension detection if it's not the default fallback
        if detected != "rust" || path.ends_with(".rs") {
            return detected;
        }
    }

    // 🥈 SECOND PRIORITY: Project type context
    if let Some(proj_type) = project_type {
        let detected = language_from_project_type(proj_type);
        if detected != "rust" {
            return detected;
        }
    }

    // 🥉 THIRD PRIORITY: Content pattern analysis
    language_from_content(content)
}

/// 📋 REQUIREMENT EXTRACTION: Extract development requirements from task content
/// WHY SEPARATE: Pattern matching logic reused across multiple agent types
/// FUTURE: Consider more sophisticated NLP-based requirement extraction
pub fn extract_requirements_from_content(
    content: &str,
    context: &HashMap<String, String>,
) -> Vec<String> {
    let mut requirements = Vec::new();
    let content_lower = content.to_lowercase();

    // 🧪 TESTING REQUIREMENTS: Detect test-related requests
    if content_lower.contains("test") {
        requirements.push("Include unit tests".to_string());
    }

    // 🌐 API REQUIREMENTS: Detect API development needs
    if content_lower.contains("api") {
        requirements.push("Follow REST API conventions".to_string());
    }

    // ⚡ ASYNC REQUIREMENTS: Detect async programming needs
    if content_lower.contains("async") {
        requirements.push("Use async/await pattern".to_string());
    }

    // 🚨 ERROR HANDLING: Detect error handling emphasis
    if content_lower.contains("error") {
        requirements.push("Implement comprehensive error handling".to_string());
    }

    // 📝 CODING STANDARDS: Apply context-specific standards
    if let Some(coding_standards) = context.get("coding_standards") {
        requirements.push(format!("Follow coding standards: {coding_standards}"));
    }

    // 🏗️ DEFAULT REQUIREMENTS: Apply when no specific requirements detected
    // Why: Ensures consistent quality baseline for all generated code
    if requirements.is_empty() {
        requirements.push("Follow SOLID principles".to_string());
        requirements.push("Apply DRY principle".to_string());
        requirements.push("Use SID naming conventions".to_string());
    }

    requirements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(language_from_extension("main.rs"), "rust");
        assert_eq!(language_from_extension("script.py"), "python");
        assert_eq!(language_from_extension("app.ts"), "typescript");
        assert_eq!(language_from_extension("unknown.xyz"), "rust"); // fallback
    }

    #[test]
    fn test_language_from_project_type() {
        assert_eq!(language_from_project_type("cargo"), "rust");
        assert_eq!(language_from_project_type("npm"), "javascript");
        assert_eq!(language_from_project_type("unknown"), "rust"); // fallback
    }

    #[test]
    fn test_language_from_content() {
        assert_eq!(language_from_content("use tokio in rust"), "rust");
        assert_eq!(language_from_content("import numpy as np"), "python");
        assert_eq!(language_from_content("generic content"), "rust"); // fallback
    }

    #[test]
    fn test_smart_detection() {
        // File extension takes priority
        assert_eq!(
            detect_language_from_context(Some("app.py"), Some("node"), "rust code"),
            "python"
        );

        // Project type when no clear file extension
        assert_eq!(
            detect_language_from_context(Some("unknown.xyz"), Some("npm"), "rust code"),
            "javascript"
        );

        // Content analysis as fallback
        assert_eq!(
            detect_language_from_context(None, Some("unknown"), "python import"),
            "python"
        );
    }
}
