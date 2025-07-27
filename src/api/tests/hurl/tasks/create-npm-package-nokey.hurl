# source .env && hurl --variable API_KEY="$API_KEY" --variable BASE_URL="$BASE_URL"  src/api/tests/hurl/tasks/create-npm-package-nokey.hurl

POST {{BASE_URL}}/tasks
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a TypeScript NPM Package for Advanced Number Formatting\nBuild a lightweight NPM package that formats numbers using progressive abbreviations:\nCore Requirements:\n\nFormat numbers with standard prefixes: K (kilo), M (million), B (billion), T (trillion), Q (quadrillion), S (sextillion), etc.\nFor extremely large numbers, combine prefixes: \"10.2KM\" (10,200 million), \"4.12MQ\" (4.12 million quadrillion)\nConfigurable decimal precision\nHandle negative numbers and edge cases\nTypeScript with full type safety\n\nPackage Specs:\n\nZero dependencies\nDual ESM/CommonJS support\nComprehensive test suite\nComplete documentation with examples\nOptimized bundle size\nCI/CD pipeline ready\n\nAPI:\n\nMain format function with options interface\nReverse parsing capability (string back to number)\nCustomizable precision and rounding behavior\n\nThe package should efficiently handle numbers from thousands to astronomically large values using the combined prefix system when single prefixes are insufficient.",
  "priority": "Medium",
  "context": {
    "language": "typescript"
  }
}