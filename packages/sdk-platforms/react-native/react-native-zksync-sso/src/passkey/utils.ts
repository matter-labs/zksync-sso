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