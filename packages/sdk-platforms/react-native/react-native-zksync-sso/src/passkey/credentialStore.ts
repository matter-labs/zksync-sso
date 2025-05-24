export type AuthenticatorTransport = "ble" | "hybrid" | "internal" | "nfc" | "usb";

export type PublicKeyCredentialType = "public-key";

export interface StoredPasskeyCredential {
    id: string;
    type: PublicKeyCredentialType;
    transports: AuthenticatorTransport[]
}

let storedCredentials: StoredPasskeyCredential[] = [];

export const addCredential = (credential: StoredPasskeyCredential): void => {
    console.log("addCredential - credential: ", credential);
    // Avoid duplicates
    if (!storedCredentials.find(c => c.id === credential.id)) {
        storedCredentials.push(credential);
        console.log("addCredential - credential added");
    }
};

export const getCredentials = (): StoredPasskeyCredential[] => {
    return [...storedCredentials]; // Return a copy
};

export const clearCredentials = (): void => {
    storedCredentials = [];
}; 