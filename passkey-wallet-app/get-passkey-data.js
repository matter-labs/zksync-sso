// Run this in the browser console to get your passkey data
console.log('Passkey data from localStorage:');
const passkeyData = JSON.parse(localStorage.getItem('zksync-sso-passkey'));
console.log('Credential ID:', passkeyData.credentialId);
console.log('Public Key (array):', passkeyData.credentialPublicKey);

// Extract x and y coordinates
const publicKeyBytes = new Uint8Array(passkeyData.credentialPublicKey);
const x = '0x' + Array.from(publicKeyBytes.slice(-64, -32)).map(b => b.toString(16).padStart(2, '0')).join('');
const y = '0x' + Array.from(publicKeyBytes.slice(-32)).map(b => b.toString(16).padStart(2, '0')).join('');

console.log('\nPaste these into initialize-account.js:');
console.log(`const CREDENTIAL_ID = '${passkeyData.credentialId}';`);
console.log(`const PUBLIC_KEY_X = '${x}';`);
console.log(`const PUBLIC_KEY_Y = '${y}';`);
console.log(`const ORIGIN = '${window.location.origin}';`);
