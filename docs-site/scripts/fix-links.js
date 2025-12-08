const fs = require('fs');
const path = require('path');

const docsDir = path.resolve(__dirname, '../../docs');

let fixCount = 0;

// Comprehensive link fixes
const linkFixes = [
    // Double-core path in client docs
    { from: /\.\.\/core\/core\/guides\//g, to: '../core/guides/' },

    // Broken upward paths from core/getting-started
    { from: /\.\/\.\.\/\.\.\/sync\/index\.md/g, to: '../sync/index.md' },
    { from: /\.\/\.\.\/\.\.\/client\/index\.md/g, to: '../client/index.md' },

    // Broken guide paths in getting-started  
    { from: /\.\/core\/guides\/sending-messages\.md/g, to: './guides/sending-messages.md' },

    // Guides pointing to ../client from core/guides
    { from: /\.\.\/client\/index\.md/g, to: '../../client/index.md' },
    { from: /\.\.\/sync\/index\.md/g, to: '../../sync/index.md' },

    // Non-existent files in sync docs - remove or stub
    { from: /\.\/guides\/authorization\.md/g, to: '../core/guides/mutations.md' },
    { from: /\.\/guides\/exclusive-control\.md/g, to: '../core/guides/server-setup.md' },

    // Fix example links that might break
    { from: /\.\.\/CHANGELOG\.md/g, to: '../../CHANGELOG.md' },
];

function getAllFiles(dir) {
    const files = [];
    const entries = fs.readdirSync(dir, { withFileTypes: true });

    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...getAllFiles(fullPath));
        } else if (entry.name.endsWith('.md')) {
            files.push(fullPath);
        }
    }
    return files;
}

function fixLinks(filePath) {
    let content = fs.readFileSync(filePath, 'utf8');
    let modified = false;

    for (const fix of linkFixes) {
        if (fix.from.test(content)) {
            content = content.replace(fix.from, fix.to);
            modified = true;
            fixCount++;
            console.log(`Fixed: ${fix.from.source} -> ${fix.to} in ${path.relative(docsDir, filePath)}`);
        }
        fix.from.lastIndex = 0;
    }

    if (modified) {
        fs.writeFileSync(filePath, content);
    }

    return modified;
}

console.log(`Fixing links in ${docsDir}...`);

const files = getAllFiles(docsDir);
let filesModified = 0;

for (const file of files) {
    if (fixLinks(file)) {
        filesModified++;
    }
}

console.log(`\nFixed ${fixCount} links in ${filesModified} files.`);
