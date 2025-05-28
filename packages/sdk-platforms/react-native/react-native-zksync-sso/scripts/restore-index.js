#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const indexPath = 'src/index.tsx';

console.log('Restoring index.tsx using git...');

try {
    // Check if the file exists
    if (!fs.existsSync(indexPath)) {
        console.log('⚠️  index.tsx does not exist, skipping git restore');
        process.exit(0);
    }

    // Check if we're in a git repository
    try {
        execSync('git rev-parse --git-dir', { stdio: 'ignore' });
    } catch (error) {
        console.log('⚠️  Not in a git repository, skipping git restore');
        process.exit(0);
    }

    // Check if the file has changes
    try {
        execSync(`git diff --quiet HEAD -- ${indexPath}`, { stdio: 'ignore' });
        console.log('✅ index.tsx is already up to date with git');
        process.exit(0);
    } catch (error) {
        // File has changes, continue with restore
    }

    // Restore the file from git
    execSync(`git checkout HEAD -- ${indexPath}`, { stdio: 'inherit' });
    console.log('✅ index.tsx restored from git successfully!');

} catch (error) {
    console.error('❌ Error restoring index.tsx from git:', error.message);
    process.exit(1);
} 