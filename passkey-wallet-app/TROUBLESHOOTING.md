# Troubleshooting Guide - Passkey Wallet App

## "Nothing happens when I click Create Passkey"

### Step 1: Check Browser Console

1. Open your browser's Developer Tools (F12 or Cmd+Option+I on Mac)
2. Go to the "Console" tab
3. Click "Create Passkey" again
4. Look for error messages or logs

### Step 2: Common Issues & Solutions

#### Issue: "NotAllowedError: The operation either timed out or was not allowed"

**Cause**: WebAuthn requires user gesture or secure context

**Solutions**:

- Make sure you're on `http://localhost:3000` (not `http://127.0.0.1:3000`)
- Try using `localhost` instead of `127.0.0.1`
- Ensure you clicked the button (not automated/script)
- Check if your browser allows WebAuthn on localhost

#### Issue: "NotSupportedError: The user agent does not support public key credentials"

**Cause**: Browser doesn't support WebAuthn

**Solutions**:

- Update your browser to latest version
- Use Chrome 90+, Safari 14+, or Firefox 93+
- Enable experimental features if on older browser

#### Issue: "No authenticator available"

**Cause**: No biometric device or security key available

**Solutions**:

- **Mac**: Enable Touch ID in System Preferences
- **Windows**: Set up Windows Hello
- **iPhone/iPad**: Enable Face ID or Touch ID
- **Android**: Enable fingerprint or face unlock
- Alternatively, use a physical security key (YubiKey, etc.)

#### Issue: "InvalidStateError: The authenticator was previously registered"

**Cause**: You already created a passkey for this app

**Solutions**:

- The passkey already exists - this is actually OK!
- Go to your browser's password manager and delete the old passkey if you want
  to start fresh
- **Chrome**: Settings → Password Manager → Passkeys
- **Safari**: Settings → Passwords → [Your Site] → Delete Passkey

#### Issue: "NotReadableError: An unknown error occurred"

**Cause**: Platform authenticator is having issues

**Solutions**:

- Restart your browser
- Restart your computer
- Try a different browser
- Check if your biometric hardware is working (try unlocking your device)

### Step 3: Verify Environment

#### Check HTTPS/Localhost

```javascript
// Run this in browser console
console.log("Protocol:", window.location.protocol); // Should be "http:" or "https:"
console.log("Hostname:", window.location.hostname); // Should be "localhost"
console.log("Port:", window.location.port); // Should be "3000"
```

✅ Good: `http://localhost:3000` ❌ Bad: `http://127.0.0.1:3000` (might not work
on some browsers) ❌ Bad: `http://192.168.1.x:3000` (won't work - needs HTTPS)

#### Check WebAuthn Support

```javascript
// Run this in browser console
if (window.PublicKeyCredential) {
  console.log("✅ WebAuthn is supported!");

  PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable().then(
    (available) => {
      if (available) {
        console.log("✅ Platform authenticator (biometrics) available!");
      } else {
        console.log("❌ No platform authenticator found");
        console.log("   → Enable Touch ID / Face ID / Windows Hello");
      }
    },
  );
} else {
  console.log("❌ WebAuthn NOT supported in this browser");
  console.log("   → Update your browser or try Chrome/Safari");
}
```

### Step 4: Browser-Specific Issues

#### Chrome/Edge

- Make sure you're on version 90+
- Enable flags if needed: `chrome://flags/#enable-web-authentication-api`
- Check: Settings → Privacy and security → Site settings → Permissions →
  Passkeys

#### Safari (Mac)

- macOS 13+ (Ventura or later) recommended
- Enable Touch ID: System Preferences → Touch ID & Password
- Allow website to use Touch ID when prompted

#### Firefox

- Version 93+ required
- May need to enable: `about:config` → `security.webauthn.enable_uv_token` →
  true
- Some features may be limited compared to Chrome/Safari

### Step 5: Debug Mode

If the issue persists, check the detailed console logs:

1. Open Console (F12)
2. Click "Create Passkey"
3. You should see:

   ```
   Creating passkey...
   User name: YourName
   Hostname: localhost
   Origin: http://localhost:3000
   Challenge generated
   Calling WebAuthn startRegistration...
   Registration options: {...}
   ```

4. Look for where it stops:
   - **Stops after "Calling WebAuthn..."**: Browser/system issue
   - **Shows error**: Read the error message
   - **No logs at all**: JavaScript error - check for exceptions

### Step 6: Alternative Testing

If localhost isn't working, try deploying to a service with HTTPS:

```bash
# Build the app
npm run build

# Deploy to Vercel (free)
npx vercel --prod

# Or deploy to Netlify
npx netlify deploy --prod --dir=dist
```

Then access via the HTTPS URL provided.

### Step 7: System Requirements Checklist

- [ ] Modern OS (macOS 13+, Windows 10+, iOS 14+, Android 9+)
- [ ] Modern browser (Chrome 90+, Safari 14+, Firefox 93+)
- [ ] HTTPS **OR** localhost
- [ ] Biometric authentication enabled on device
- [ ] Browser permissions granted for WebAuthn
- [ ] No corporate policies blocking WebAuthn

## Still Not Working?

### Check Console for Specific Errors

Look for these patterns:

#### Pattern 1: Permission Denied

```
NotAllowedError: The user denied permission
```

→ You clicked "Cancel" on the biometric prompt. Try again and approve it.

#### Pattern 2: Timeout

```
NotAllowedError: Timeout
```

→ Biometric prompt timed out. Try again and respond faster.

#### Pattern 3: No Support

```
undefined is not an object (evaluating 'navigator.credentials.create')
```

→ Browser too old or WebAuthn disabled.

### Test with Simple Example

Try this minimal test in the browser console:

```javascript
navigator.credentials
  .create({
    publicKey: {
      challenge: new Uint8Array(32),
      rp: { name: "Test" },
      user: {
        id: new Uint8Array(16),
        name: "test",
        displayName: "Test User",
      },
      pubKeyCredParams: [{ alg: -7, type: "public-key" }],
      timeout: 60000,
    },
  })
  .then((cred) => {
    console.log("✅ SUCCESS! WebAuthn works!", cred);
  })
  .catch((err) => {
    console.error("❌ ERROR:", err.name, err.message);
  });
```

If this works, the app should work too.

## Getting More Help

1. **Check browser DevTools console** - Shows the exact error
2. **Try different browser** - Helps isolate browser-specific issues
3. **Check system biometrics** - Make sure you can unlock your device
4. **Clear browser data** - Sometimes helps with credential conflicts

## Common Environment Issues

### Issue: Running on IP address instead of localhost

```bash
# ❌ Don't use:
http://127.0.0.1:3000
http://192.168.1.x:3000

# ✅ Use:
http://localhost:3000
```

### Issue: Browser not allowing localhost

Some browsers are strict. Try adding to vite.config.js:

```javascript
export default defineConfig({
  server: {
    host: "localhost", // Explicitly set to localhost
    port: 3000,
    https: false, // Don't need HTTPS for localhost
  },
});
```

### Issue: Antivirus/Firewall blocking

Some security software blocks WebAuthn. Try:

- Temporarily disable antivirus
- Add localhost to whitelist
- Check browser's site permissions

## Success Checklist

When everything works, you should see:

1. ✅ Click "Create Passkey"
2. ✅ Browser shows biometric prompt (Touch ID / Face ID / Windows Hello)
3. ✅ Authenticate with your biometric
4. ✅ See "✓ Passkey Created!" message
5. ✅ Credential ID displayed
6. ✅ "Deploy Account" button enabled

## Need More Help?

- Check the browser console for detailed logs
- Try the test snippet above
- Update your browser
- Try a different device/browser
- Contact support with console error messages
