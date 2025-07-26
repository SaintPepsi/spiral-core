#!/usr/bin/env node
/**
 * Validate that all markdown document links exist and are accessible
 * Prevents broken references when reorganizing documentation
 */
import * as fs from 'fs';
import * as path from 'path';

// Types
interface MarkdownLink {
  text: string;
  path: string;
  line: number;
  source: string;
}

interface ValidationError {
  type: 'broken-link' | 'read-error';
  file: string;
  line?: number;
  link?: string;
  text?: string;
  resolved?: string;
  message: string;
}

interface ValidationWarning {
  file: string;
  line?: number;
  message: string;
}

// Global state
const errors: ValidationError[] = [];
const warnings: ValidationWarning[] = [];
const checkedFiles = new Set<string>();

/**
 * Recursively find all markdown files in a directory
 */
function findMarkdownFiles(dir: string, files: string[] = []): string[] {
  if (!fs.existsSync(dir)) {
    return files;
  }

  const items = fs.readdirSync(dir);
  
  for (const item of items) {
    const fullPath = path.join(dir, item);
    const stat = fs.statSync(fullPath);
    
    if (stat.isDirectory() && !item.startsWith('.') && item !== 'node_modules') {
      findMarkdownFiles(fullPath, files);
    } else if (stat.isFile() && item.endsWith('.md')) {
      files.push(fullPath);
    }
  }
  
  return files;
}

/**
 * Extract markdown links from file content
 */
function extractLinks(content: string, filePath: string): MarkdownLink[] {
  const links: MarkdownLink[] = [];
  
  // Match [text](path) links
  const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
  let match;
  
  while ((match = linkRegex.exec(content)) !== null) {
    const linkText = match[1];
    const linkPath = match[2];
    
    // Skip external URLs, anchors, email links, and template placeholders
    if (linkPath.startsWith('http') || linkPath.startsWith('#') || linkPath.startsWith('mailto:') || 
        linkPath === '{}' || linkPath.includes('url "Title"')) {
      continue;
    }
    
    links.push({
      text: linkText,
      path: linkPath,
      line: content.substring(0, match.index).split('\n').length,
      source: filePath
    });
  }
  
  return links;
}

/**
 * Resolve relative path based on source file location
 */
function resolvePath(linkPath: string, sourceFile: string): string {
  const sourceDir = path.dirname(sourceFile);
  return path.resolve(sourceDir, linkPath);
}

/**
 * Check if a file path exists
 */
function checkFileExists(resolvedPath: string, link: MarkdownLink): boolean {
  if (fs.existsSync(resolvedPath)) {
    return true;
  }
  
  errors.push({
    type: 'broken-link',
    file: link.source,
    line: link.line,
    link: link.path,
    text: link.text,
    resolved: resolvedPath,
    message: `Broken link: "${link.text}" -> "${link.path}"`
  });
  
  return false;
}

/**
 * Validate all links in a markdown file
 */
function validateFile(filePath: string): void {
  if (checkedFiles.has(filePath)) {
    return;
  }
  
  checkedFiles.add(filePath);
  
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    const links = extractLinks(content, filePath);
    
    for (const link of links) {
      const resolvedPath = resolvePath(link.path, filePath);
      checkFileExists(resolvedPath, link);
    }
    
  } catch (error) {
    errors.push({
      type: 'read-error',
      file: filePath,
      message: `Error reading file: ${error.message}`
    });
  }
}

/**
 * Main validation function
 */
function validateDocumentation(): number {
  console.log('🔍 Validating markdown documentation links...\n');
  
  const projectRoot = process.cwd();
  const markdownFiles = findMarkdownFiles(projectRoot);
  
  console.log(`Found ${markdownFiles.length} markdown files to validate\n`);
  
  // Validate each file
  for (const file of markdownFiles) {
    validateFile(file);
  }
  
  // Report results
  if (errors.length === 0 && warnings.length === 0) {
    console.log('✅ All documentation links are valid!');
    return 0;
  }
  
  if (warnings.length > 0) {
    console.log('⚠️  Warnings:');
    for (const warning of warnings) {
      console.log(`   ${warning.file}:${warning.line || '?'} - ${warning.message}`);
    }
    console.log();
  }
  
  if (errors.length > 0) {
    console.log('❌ Errors found:');
    for (const error of errors) {
      console.log(`   ${path.relative(projectRoot, error.file)}:${error.line || '?'} - ${error.message}`);
    }
    console.log(`\n${errors.length} broken link(s) found`);
    return 1;
  }
  
  return 0;
}

// Run validation if called directly
if (require.main === module) {
  const exitCode = validateDocumentation();
  process.exit(exitCode);
}

export { validateDocumentation };