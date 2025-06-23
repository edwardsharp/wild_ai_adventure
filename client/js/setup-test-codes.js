#!/usr/bin/env node

/**
 * Test helper script to setup known invite codes for integration tests
 * This script uses the CLI to create predictable invite codes that tests can use
 */

import { spawn } from 'child_process';
import { writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Test configuration
const TEST_INVITE_CODES = [
  'TEST001A',
  'TEST002B',
  'TEST003C',
  'TEST004D',
  'TEST005E',
  'INTEG001',
  'INTEG002',
  'INTEG003',
  'ROUTES01',
  'ROUTES02',
  'ROUTES03',
  'ROUTES04',
  'ROUTES05',
  'BURST001',
  'BURST002',
  'BURST003',
  'BURST004',
  'BURST005',
  'BURST006',
  'BURST007',
  'BURST008',
  'BURST009',
  'BURST010',
  'SECURITY01',
  'SECURITY02',
  'SPECIAL01',
  'SPECIAL02',
  'SPECIAL03',
  'SPECIAL04',
  'SPECIAL05',
];

const TEST_DB_URL =
  process.env.TEST_DB_URL ||
  'postgres://postgres:supersecret@localhost:5432/webauthn_db';
const CLI_CONFIG = process.env.CLI_CONFIG || 'assets/config/config.jsonc';
const CLI_SECRETS =
  process.env.CLI_SECRETS || 'assets/config/config.secrets.jsonc';

/**
 * Execute CLI command to create custom invite codes
 */
async function createTestInviteCodes() {
  console.log('🎫 Setting up test invite codes...');

  const customCodes = TEST_INVITE_CODES.join(',');

  const args = [
    'run',
    '--release',
    '--bin',
    'cli',
    '--',
    '--config',
    CLI_CONFIG,
    '--secrets',
    CLI_SECRETS,
    'users',
    'generate-invite',
    '--custom',
    customCodes,
  ];

  console.log(`📝 Creating codes: ${TEST_INVITE_CODES.join(', ')}`);

  return new Promise((resolve, reject) => {
    const childProcess = spawn('cargo', args, {
      cwd: join(__dirname, '..'), // Go to project root
      env: {
        ...process.env,
        DATABASE_URL: TEST_DB_URL,
        RUST_LOG: 'warn', // Reduce noise
      },
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';

    childProcess.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    childProcess.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    childProcess.on('close', (code) => {
      if (code === 0) {
        console.log('✅ Test invite codes created successfully');
        console.log(stdout);
        resolve(stdout);
      } else {
        console.error('❌ Failed to create test invite codes');
        console.error('STDOUT:', stdout);
        console.error('STDERR:', stderr);
        reject(new Error(`CLI process exited with code ${code}`));
      }
    });

    childProcess.on('error', (error) => {
      console.error('❌ Failed to spawn CLI process:', error);
      reject(error);
    });
  });
}

/**
 * Save test codes to JSON file for tests to import
 */
function saveTestCodesFile() {
  console.log('💾 Saving test codes to file...');

  const testDataDir = join(__dirname, 'test-data');
  mkdirSync(testDataDir, { recursive: true });

  const testData = {
    inviteCodes: TEST_INVITE_CODES,
    createdAt: new Date().toISOString(),
    description: 'Predictable invite codes for integration tests',
  };

  const filePath = join(testDataDir, 'test-codes.json');
  writeFileSync(filePath, JSON.stringify(testData, null, 2));

  console.log(`✅ Test codes saved to: ${filePath}`);

  // Also create a simple array file for easy importing
  const codesOnlyPath = join(testDataDir, 'invite-codes.json');
  writeFileSync(codesOnlyPath, JSON.stringify(TEST_INVITE_CODES, null, 2));

  console.log(`✅ Codes array saved to: ${codesOnlyPath}`);
}

/**
 * Main function
 */
async function main() {
  try {
    console.log('🚀 Setting up test environment...');
    console.log(`Database: ${TEST_DB_URL}`);
    console.log(`Config: ${CLI_CONFIG}`);
    console.log(`Secrets: ${CLI_SECRETS}`);
    console.log('');

    await createTestInviteCodes();
    saveTestCodesFile();

    console.log('');
    console.log('🎉 Test setup complete!');
    console.log('');
    console.log('Available test invite codes:');
    TEST_INVITE_CODES.forEach((code, index) => {
      console.log(`  ${index + 1}. ${code}`);
    });
    console.log('');
    console.log(
      '💡 Tests can now import these codes from test-data/invite-codes.json'
    );
  } catch (error) {
    console.error('❌ Setup failed:', error.message);
    process.exit(1);
  }
}

// Run the script
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { TEST_INVITE_CODES, createTestInviteCodes, saveTestCodesFile };
