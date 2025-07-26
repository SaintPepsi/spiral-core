POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Review the entire codebase against the provided coding standards document. Fix any violations you find, create a pull request with your changes, and provide a summary.\n\nSteps:\n1. Read the coding standards document at docs/CODE_REVIEW_TEMPLATES.md\n2. Scan all code files for violations\n3. Fix the issues you can automatically resolve\n4. Create a PR with your fixes\n5. Write a summary of what you found and fixed\n\nPR Title: \"Code review fixes - [brief description of main changes]\"\n\nSummary Format:\n- Files reviewed: [number]\n- Issues found: [number]\n- Issues fixed: [number]\n- Main violations: [list top 3-5 types]\n- Manual review needed: [any issues you couldn't fix]",
  "priority": "Medium",
  "context": {
    "language": "typescript"
  }
}