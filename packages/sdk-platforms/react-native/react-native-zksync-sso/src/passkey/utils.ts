// @ts-ignore
import { initAndroidLogger, initAppleLogger, testLogging, type RpId, LogLevel } from 'react-native-zksync-sso';
import { Platform } from 'react-native';

/**
 * Information about the relying party (RP) for passkey registration
 */
export interface RPInfo {
    name: string;
    id: RpId;
}

/**
 * Information about the account being registered
 */
export interface AccountInfo {
    name: string;
    userID: string;
    rp: RPInfo;
}

/**
 * Extracts the RPID string from the FFI RpId enum type
 */
export const getRpIdString = (rpId: RpId): string => {
    if (!rpId) {
        throw new Error('getRpIdString: rpId is null or undefined');
    }
    const tag = (rpId as any).tag;
    if (tag === 'Apple') {
        return (rpId as any).inner[0];
    } else if (tag === 'Android') {
        return (rpId as any).inner[0].rpId;
    }
    throw new Error(`Unknown RpId type: ${tag}. Full rpId object: ${JSON.stringify(rpId)}`);
};

/**
 * Initializes platform-specific logging for the ZkSync SSO SDK.
 * This should be called early in the app lifecycle to enable proper logging.
 * Automatically detects the platform and initializes the appropriate logger.
 * 
 * @param appleBundleIdentifier - Required for iOS logging, ignored on Android (defaults to example bundle identifier)
 */
export function initializePlatformLogger(appleBundleIdentifier: string): void {
    try {
        const logLevel = LogLevel.Trace;
        if (Platform.OS === 'ios') {
            initAppleLogger(appleBundleIdentifier, logLevel);
        } else if (Platform.OS === 'android') {
            initAndroidLogger(logLevel);
        } else {
            console.error(`Unsupported platform for logging: ${Platform.OS}`);
            return;
        }
    } catch (error) {
        console.error(`Failed to initialize ${Platform.OS} logger:`, error);
    }
}

/**
 * Converts a regular string to its base64 representation.
 * Example: "jdoe@example.com" becomes "amRvZUBleGFtcGxlLmNvbQ=="
 * 
 * @param input The string to convert to base64
 * @returns A base64-encoded string
 */
export function stringToBase64(input: string): string {
    // For React Native, we can use the built-in btoa function
    try {
        return btoa(input);
    } catch (error) {
        console.error('Error encoding string to base64:', error);
        // Fallback implementation in case btoa fails
        const bytes = new TextEncoder().encode(input);
        const binString = Array.from(bytes)
            .map(byte => String.fromCharCode(byte))
            .join('');
        return btoa(binString);
    }
}

/**
 * Converts a base64url encoded string to standard base64.
 * Base64url uses '-' instead of '+', '_' instead of '/' and omits padding.
 * 
 * @param base64url The base64url encoded string
 * @returns A standard base64 encoded string
 */
export function base64urlToBase64(base64url: string): string {
    // Replace - with +, _ with /, and add padding if needed
    let base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');

    // Add padding if necessary
    const padding = base64.length % 4;
    if (padding) {
        base64 += '='.repeat(4 - padding);
    }

    return base64;
}

/**
 * Decodes a base64url string to a Uint8Array of bytes.
 * 
 * @param base64url The base64url encoded string
 * @returns A Uint8Array containing the decoded bytes
 */
export function base64urlToBytes(base64url: string): Uint8Array {
    const base64 = base64urlToBase64(base64url);
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
}

/**
 * Encodes a Uint8Array of bytes to a standard base64 string.
 * 
 * @param bytes The Uint8Array to encode
 * @returns A standard base64 encoded string
 */
export function bytesToBase64(bytes: Uint8Array): string {
    const binaryString = Array.from(bytes)
        .map(byte => String.fromCharCode(byte))
        .join('');
    return btoa(binaryString);
}


/**
 * Converts an ArrayBuffer to a base64url string (compatible with Swift's toBase64URLEncodedString())
 * 
 * @param buffer The ArrayBuffer to convert
 * @returns A base64url-encoded string
 */
export function arrayBufferToBase64Url(buffer: ArrayBuffer): string {
    // Convert ArrayBuffer to binary string
    const bytes = new Uint8Array(buffer);
    const binaryString = Array.from(bytes)
        .map(byte => String.fromCharCode(byte))
        .join('');

    // Convert binary string to base64
    const base64 = btoa(binaryString);

    // Convert base64 to base64url
    // - Replace + with -
    // - Replace / with _
    // - Remove trailing = padding
    return base64
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=+$/, '');
}

/**
 * Converts a base64url string (as encoded by Swift's toBase64URLEncodedString())
 * to an ArrayBuffer. WebAuthn values from Swift typically come in this format.
 * 
 * Base64URL encoding differences from standard base64:
 * - Uses '-' instead of '+'
 * - Uses '_' instead of '/'
 * - Omits padding '=' characters
 */
export function base64ToArrayBuffer(base64Input: string): ArrayBuffer {
    // Convert base64url to base64 if necessary
    // - Replace - with +
    // - Replace _ with /
    // - Add padding if needed
    let base64 = base64Input
        .replace(/-/g, '+')
        .replace(/_/g, '/');

    // Add padding if needed to make length a multiple of 4
    const paddingLength = (4 - (base64.length % 4)) % 4;
    base64 += '='.repeat(paddingLength);

    // Convert base64 to binary string
    const binaryString = atob(base64);

    // Convert binary string to Uint8Array
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }

    return bytes.buffer;
}

/**
 * Converts an ArrayBuffer to a hexadecimal string representation.
 * 
 * @param buffer The ArrayBuffer to convert
 * @returns A hex string (e.g., "48656c6c6f")
 */
export function arrayBufferToHexString(buffer: ArrayBuffer): string {
    return Array.from(new Uint8Array(buffer))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
}