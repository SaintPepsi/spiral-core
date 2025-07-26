#!/usr/bin/env python3
"""
Convert cargo-audit JSON output to SARIF format for GitHub Security integration
"""
import json
import sys
from typing import Dict, List, Any

def convert_severity(severity: str) -> str:
    """Convert RustSec severity to SARIF level"""
    severity_map = {
        "critical": "error",
        "high": "error",
        "medium": "warning",
        "low": "note",
        "informational": "note"
    }
    return severity_map.get(severity.lower(), "warning")

def create_sarif_result(vuln: Dict[str, Any]) -> Dict[str, Any]:
    """Create a SARIF result from a vulnerability"""
    advisory = vuln.get("advisory", {})
    package = vuln.get("package", {})
    
    return {
        "ruleId": advisory.get("id", "UNKNOWN"),
        "level": convert_severity(vuln.get("severity", "medium")),
        "message": {
            "text": advisory.get("description", "Unknown vulnerability")
        },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": {
                    "uri": "Cargo.toml"
                },
                "region": {
                    "startLine": 1,
                    "startColumn": 1
                }
            }
        }],
        "partialFingerprints": {
            "packageName": package.get("name", ""),
            "packageVersion": package.get("version", "")
        },
        "properties": {
            "security-severity": vuln.get("cvss", {}).get("score", 5.0),
            "tags": ["security", "dependency", "vulnerability"],
            "precision": "very-high"
        }
    }

def create_sarif_rule(vuln: Dict[str, Any]) -> Dict[str, Any]:
    """Create a SARIF rule from a vulnerability"""
    advisory = vuln.get("advisory", {})
    
    return {
        "id": advisory.get("id", "UNKNOWN"),
        "name": advisory.get("title", "Unknown vulnerability"),
        "shortDescription": {
            "text": advisory.get("title", "Unknown vulnerability")
        },
        "fullDescription": {
            "text": advisory.get("description", "")
        },
        "help": {
            "text": f"Update {vuln.get('package', {}).get('name', 'package')} to a patched version",
            "markdown": f"## {advisory.get('title', 'Vulnerability')}\n\n{advisory.get('description', '')}\n\n### References\n- {advisory.get('url', '')}"
        },
        "properties": {
            "security-severity": vuln.get("cvss", {}).get("score", 5.0),
            "tags": ["security", "vulnerability"]
        }
    }

def convert_audit_to_sarif(audit_data: Dict[str, Any]) -> Dict[str, Any]:
    """Convert cargo-audit JSON to SARIF format"""
    vulnerabilities = audit_data.get("vulnerabilities", {}).get("list", [])
    
    sarif = {
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "cargo-audit",
                    "version": "0.18.0",
                    "informationUri": "https://github.com/RustSec/rustsec",
                    "rules": []
                }
            },
            "results": [],
            "columnKind": "utf16CodeUnits"
        }]
    }
    
    # Add rules and results for each vulnerability
    for vuln in vulnerabilities:
        sarif["runs"][0]["tool"]["driver"]["rules"].append(create_sarif_rule(vuln))
        sarif["runs"][0]["results"].append(create_sarif_result(vuln))
    
    return sarif

def main():
    if len(sys.argv) != 3:
        print("Usage: audit-to-sarif.py <input.json> <output.sarif>")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    
    try:
        with open(input_file, 'r') as f:
            audit_data = json.load(f)
        
        sarif_data = convert_audit_to_sarif(audit_data)
        
        with open(output_file, 'w') as f:
            json.dump(sarif_data, f, indent=2)
        
        print(f"Successfully converted {input_file} to {output_file}")
        
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()