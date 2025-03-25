export function createSecretAccountSalt(): string {
    const zeroArray = new Uint8Array(32);
    const base64 = btoa(String.fromCharCode.apply(null, Array.from(zeroArray)));
    console.log('Using secret account salt:', base64);
    return base64;
}