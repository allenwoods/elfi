#!/usr/bin/env node

// MCP Configuration Validator for ELFI Context7
const fs = require('fs');
const path = require('path');

console.log('Validating MCP Configuration for ELFI Context7...\n');

// Check for mcp.json
const mcpConfigPath = path.join(__dirname, 'mcp.json');
if (!fs.existsSync(mcpConfigPath)) {
    console.error('✗ mcp.json not found');
    process.exit(1);
}

try {
    const config = JSON.parse(fs.readFileSync(mcpConfigPath, 'utf8'));
    
    // Validate Context7 server configuration
    if (!config.mcpServers || !config.mcpServers.context7) {
        throw new Error('Context7 server configuration missing');
    }
    
    const context7 = config.mcpServers.context7;
    
    // Check required fields
    const requiredFields = ['command', 'args', 'env', 'settings'];
    for (const field of requiredFields) {
        if (!context7[field]) {
            throw new Error(`Missing required field: ${field}`);
        }
    }
    
    console.log('✓ Context7 server configuration valid');
    
    // Check capabilities
    const caps = context7.settings.capabilities;
    console.log('\nCapabilities:');
    console.log(`  Tools: ${caps.tools ? '✓' : '✗'}`);
    console.log(`  Resources: ${caps.resources ? '✓' : '✗'}`);
    console.log(`  Prompts: ${caps.prompts ? '✓' : '✗'}`);
    console.log(`  Sampling: ${caps.sampling ? '✓' : '✗'}`);
    
    // Check integrations
    const integration = context7.settings.integration;
    console.log('\nIntegrations:');
    console.log(`  Zenoh: ${integration.zenoh?.enabled ? '✓' : '✗'}`);
    console.log(`  CRDT: ${integration.crdt?.enabled ? '✓' : '✗'}`);
    console.log(`  Storage: ${integration.storage?.backend || 'not configured'}`);
    
    // Check other servers
    console.log('\nAdditional MCP Servers:');
    for (const [name, server] of Object.entries(config.mcpServers)) {
        if (name !== 'context7') {
            console.log(`  ${name}: ${server.command ? '✓' : '✗'}`);
        }
    }
    
    // Check environment variables
    console.log('\nEnvironment Configuration:');
    const envPath = path.join(__dirname, '.env');
    if (fs.existsSync(envPath)) {
        const envContent = fs.readFileSync(envPath, 'utf8');
        const hasContext7Key = envContent.includes('CONTEXT7_API_KEY=') && 
                               !envContent.includes('CONTEXT7_API_KEY=your_context7_api_key_here');
        console.log(`  .env file: ✓`);
        console.log(`  Context7 API key: ${hasContext7Key ? '✓ configured' : '⚠ needs configuration'}`);
    } else {
        console.log(`  .env file: ✗ (copy from .env.example)`);
    }
    
    console.log('\n✅ MCP configuration validation complete!');
    console.log('\nTo activate:');
    console.log('1. Ensure .env is configured with your API keys');
    console.log('2. Restart Claude Code to load the MCP configuration');
    console.log('3. The Context7 server will be available in your Claude Code session');
    
} catch (error) {
    console.error(`\n✗ Validation failed: ${error.message}`);
    process.exit(1);
}